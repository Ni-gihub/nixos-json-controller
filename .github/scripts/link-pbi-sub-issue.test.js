"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const {
  extractParentPbiValue,
  parseIssueNumber,
  currentIssueType,
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

test("does not call GitHub when no parent was requested", async () => {
  const core = makeCore();
  const result = await linkIssueToPbi({
    github: { rest: { issues: { get: async () => { throw new Error("unexpected API call"); } } } },
    context: {
      repo: { owner: "nxc-project", repo: "nixos-json-controller" },
      payload: { issue: { number: 10, body: "### Parent PBI number (optional)\n" } },
    },
    core,
  });
  assert.equal(result.reason, "no-parent-requested");
});

test("does not link when the parent number is invalid", async () => {
  const core = makeCore();
  const result = await linkIssueToPbi({
    github: { rest: { issues: { get: async () => { throw new Error("unexpected API call"); } } } },
    context: {
      repo: { owner: "nxc-project", repo: "nixos-json-controller" },
      payload: { issue: { number: 10, body: "### Parent PBI number (optional)\n177 extra" } },
    },
    core,
  });
  assert.equal(result.reason, "invalid-parent-number");
  assert.equal(core.logs.warnings.length, 1);
});

test("links a supported child when the parent is a PBI and no current parent exists", async () => {
  const core = makeCore();
  const calls = [];
  const issues = {
    20: { id: 2020, number: 20, type: { name: "Task" } },
    177: { id: 1770, number: 177, type: { name: "PBI" } },
  };
  const github = {
    rest: {
      issues: {
        get: async ({ issue_number }) => ({ data: issues[issue_number] }),
      },
    },
    request: async (route, params) => {
      calls.push({ route, params });
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
    context: {
      repo: { owner: "nxc-project", repo: "nixos-json-controller" },
      payload: {
        issue: { number: 20, body: "### Parent PBI number (optional)\n177" },
      },
    },
    core,
  });
  assert.equal(result.action, "linked");
  assert.equal(result.parentNumber, 177);
  assert.equal(result.childNumber, 20);
  assert.equal(calls.length, 2);
  assert.equal(calls[1].route, "POST /repos/{owner}/{repo}/issues/{issue_number}/sub_issues");
  assert.equal(calls[1].params.issue_number, 177);
  assert.equal(calls[1].params.sub_issue_id, 2020);
});

test("refuses a parent whose Issue Type is not PBI", async () => {
  const core = makeCore();
  const issues = {
    20: { id: 2020, number: 20, type: { name: "Task" } },
    177: { id: 1770, number: 177, type: { name: "Project" } },
  };
  const github = {
    rest: { issues: { get: async ({ issue_number }) => ({ data: issues[issue_number] }) } },
    request: async () => { throw new Error("must not create a relationship"); },
  };
  const result = await linkIssueToPbi({
    github,
    context: {
      repo: { owner: "nxc-project", repo: "nixos-json-controller" },
      payload: { issue: { number: 20, body: "### Parent PBI number (optional)\n177" } },
    },
    core,
  });
  assert.equal(result.reason, "parent-is-not-pbi");
});

test("never silently replaces a different existing parent", async () => {
  const core = makeCore();
  const issues = {
    20: { id: 2020, number: 20, type: { name: "Task" } },
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
    context: {
      repo: { owner: "nxc-project", repo: "nixos-json-controller" },
      payload: { issue: { number: 20, body: "### Parent PBI number (optional)\n177" } },
    },
    core,
  });
  assert.equal(result.reason, "different-parent-exists");
  assert.equal(core.logs.warnings.length, 1);
});
