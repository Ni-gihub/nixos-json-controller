"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const {
  extractParentPbiValue,
  parseIssueNumber,
  currentIssueType,
  requestedParentNumber,
  linkIssueToPbi,
} = require("./link-pbi-sub-issue.js");

function makeCore() {
  const logs = { info: [], warnings: [] };
  return {
    logs,
    info(message) { logs.info.push(message); },
    warning(message) { logs.warnings.push(message); },
  };
}

function contextFor(issue, body = "") {
  return {
    repo: { owner: "nxc-project", repo: "nixos-json-controller" },
    payload: {
      issue: {
        author_association: "COLLABORATOR",
        number: issue,
        body,
      },
    },
  };
}

test("extracts only the value from the Parent PBI number form field", () => {
  assert.equal(
    extractParentPbiValue("### Parent PBI number (optional)\n177\n\n### Why\nBecause"),
    "177",
  );
  assert.equal(
    extractParentPbiValue("### Parent PBI number (optional)\n\n### Why\nBecause"),
    "",
  );
  assert.equal(extractParentPbiValue("### Why\nBecause"), null);
});

test("accepts only a positive safe integer issue number", () => {
  assert.equal(parseIssueNumber("177"), 177);
  assert.equal(parseIssueNumber("1"), 1);
  assert.equal(parseIssueNumber("0"), null);
  assert.equal(parseIssueNumber("-177"), null);
  assert.equal(parseIssueNumber("177 extra"), null);
  assert.equal(parseIssueNumber("9007199254740992"), null);
});

test("reads both common GitHub Issue Type payload shapes", () => {
  assert.equal(currentIssueType({ type: "PBI" }), "PBI");
  assert.equal(currentIssueType({ type: { name: "Task" } }), "Task");
  assert.equal(currentIssueType({ type: null }), null);
});

test("parses the current Issue body into a requested parent number", () => {
  const core = makeCore();
  assert.deepEqual(
    requestedParentNumber({ number: 20, body: "### Parent PBI number (optional)\n177" }, core),
    { parentNumber: 177, reason: null },
  );
  assert.deepEqual(
    requestedParentNumber({ number: 20, body: "### Parent PBI number (optional)\n" }, core),
    { parentNumber: null, reason: "no-parent-requested" },
  );
});

test("does not let an untrusted author mutate issue hierarchy", async () => {
  const core = makeCore();
  const context = contextFor(20, "### Parent PBI number (optional)\n177");
  context.payload.issue.author_association = "NONE";
  const result = await linkIssueToPbi({
    github: { rest: { issues: { get: async () => { throw new Error("unexpected API call"); } } } },
    context,
    core,
  });
  assert.equal(result.reason, "untrusted-author");
  assert.equal(core.logs.warnings.length, 1);
});

test("uses the current API body, not a stale edited-event snapshot", async () => {
  const core = makeCore();
  const calls = [];
  const issues = {
    20: { id: 2020, number: 20, body: "### Parent PBI number (optional)\n180", type: { name: "Task" } },
    180: { id: 1800, number: 180, type: { name: "PBI" } },
  };
  const github = {
    rest: {
      issues: {
        get: async ({ issue_number }) => {
          calls.push(["get-issue", issue_number]);
          return { data: issues[issue_number] };
        },
      },
    },
    request: async (route, params) => {
      calls.push([route, params]);
      if (route.startsWith("GET ")) {
        const error = new Error("no parent");
        error.status = 404;
        throw error;
      }
      return { data: {} };
    },
  };

  const result = await linkIssueToPbi({
    github,
    context: contextFor(20, "### Parent PBI number (optional)\n177"),
    core,
  });

  assert.deepEqual(result, { action: "linked", parentNumber: 180, childNumber: 20 });
  assert.deepEqual(calls.filter((call) => call[0] === "get-issue").map((call) => call[1]), [20, 180, 20]);
  const post = calls.find((call) => call[0].startsWith("POST "));
  assert.equal(post[1].issue_number, 180);
  assert.equal(post[1].sub_issue_id, 2020);
});

test("does not link to a parent when the requested number changes during processing", async () => {
  const core = makeCore();
  const calls = [];
  let childReads = 0;
  const issues = {
    177: { id: 1770, number: 177, type: { name: "PBI" } },
    180: { id: 1800, number: 180, type: { name: "PBI" } },
  };
  const github = {
    rest: {
      issues: {
        get: async ({ issue_number }) => {
          calls.push(["get-issue", issue_number]);
          if (issue_number === 20) {
            childReads += 1;
            return {
              data: {
                id: 2020,
                number: 20,
                type: { name: "Task" },
                body: `### Parent PBI number (optional)\n${childReads === 1 ? "177" : "180"}`,
              },
            };
          }
          return { data: issues[issue_number] };
        },
      },
    },
    request: async (route, params) => {
      calls.push([route, params]);
      if (route.startsWith("GET ")) {
        const error = new Error("no parent");
        error.status = 404;
        throw error;
      }
      return { data: {} };
    },
  };

  const result = await linkIssueToPbi({
    github,
    context: contextFor(20, "### Parent PBI number (optional)\n177"),
    core,
  });

  assert.equal(result.reason, "parent-request-changed");
  assert.equal(calls.some((call) => call[0].startsWith("POST ")), false);
});

test("does not link when no parent is currently requested", async () => {
  const core = makeCore();
  const github = {
    rest: {
      issues: {
        get: async ({ issue_number }) => ({
          data: {
            id: 2020,
            number: issue_number,
            body: "### Parent PBI number (optional)\n",
            type: { name: "Task" },
          },
        }),
      },
    },
  };
  const result = await linkIssueToPbi({
    github,
    context: contextFor(20, "### Parent PBI number (optional)\n177"),
    core,
  });
  assert.equal(result.reason, "no-parent-requested");
});

test("does not link when the current parent number is invalid", async () => {
  const core = makeCore();
  const github = {
    rest: {
      issues: {
        get: async ({ issue_number }) => ({
          data: {
            id: 2020,
            number: issue_number,
            body: "### Parent PBI number (optional)\n177 extra",
            type: { name: "Task" },
          },
        }),
      },
    },
  };
  const result = await linkIssueToPbi({
    github,
    context: contextFor(20, "### Parent PBI number (optional)\n177"),
    core,
  });
  assert.equal(result.reason, "invalid-parent-number");
  assert.equal(core.logs.warnings.length, 1);
});

test("refuses a parent whose Issue Type is not PBI", async () => {
  const core = makeCore();
  const issues = {
    20: { id: 2020, number: 20, body: "### Parent PBI number (optional)\n177", type: { name: "Task" } },
    177: { id: 1770, number: 177, type: { name: "Project" } },
  };
  const github = {
    rest: { issues: { get: async ({ issue_number }) => ({ data: issues[issue_number] }) } },
    request: async () => { throw new Error("must not create a relationship"); },
  };
  const result = await linkIssueToPbi({
    github,
    context: contextFor(20, "### Parent PBI number (optional)\n177"),
    core,
  });
  assert.equal(result.reason, "parent-is-not-pbi");
});

test("does not link if the parent number changes while checking the current parent", async () => {
  const core = makeCore();
  const currentBody = "### Parent PBI number (optional)\\n177";
  const updatedBody = "### Parent PBI number (optional)\\n180";
  let body = currentBody;
  let childReads = 0;
  const calls = [];
  const issues = {
    177: { id: 1770, number: 177, type: { name: "PBI" } },
    180: { id: 1800, number: 180, type: { name: "PBI" } },
  };
  const github = {
    rest: {
      issues: {
        get: async ({ issue_number }) => {
          calls.push(["get-issue", issue_number]);
          if (issue_number === 20) {
            childReads += 1;
            return {
              data: {
                id: 2020,
                number: 20,
                body,
                type: { name: "Task" },
              },
            };
          }
          return { data: issues[issue_number] };
        },
      },
    },
    request: async (route, params) => {
      calls.push([route, params]);
      if (route.startsWith("GET ")) {
        // Simulate editing the Issue while waiting for GitHub's parent lookup.
        body = updatedBody;
        const error = new Error("no parent");
        error.status = 404;
        throw error;
      }
      return { data: {} };
    },
  };

  const result = await linkIssueToPbi({
    github,
    context: contextFor(20, currentBody),
    core,
  });

  assert.equal(result.reason, "parent-request-changed-after-parent-check");
  assert.equal(calls.some((call) => call[0].startsWith("POST ")), false);
  assert.equal(childReads, 3);
});

test("never silently replaces a different existing parent", async () => {
  const core = makeCore();
  const issues = {
    20: { id: 2020, number: 20, body: "### Parent PBI number (optional)\n177", type: { name: "Task" } },
    177: { id: 1770, number: 177, type: { name: "PBI" } },
  };
  const github = {
    rest: { issues: { get: async ({ issue_number }) => ({ data: issues[issue_number] }) } },
    request: async (route) => {
      if (route.startsWith("GET ")) return { data: { number: 180 } };
      throw new Error("must not replace an existing parent");
    },
  };
  const result = await linkIssueToPbi({
    github,
    context: contextFor(20, "### Parent PBI number (optional)\n177"),
    core,
  });
  assert.equal(result.reason, "different-parent-exists");
  assert.equal(core.logs.warnings.length, 1);
});
