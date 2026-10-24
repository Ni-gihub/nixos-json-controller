"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const { extractH2Headings, validatePullRequestBody } = require("./validate-pr-template.js");

const template = [
  "## Why",
  "<!-- Remove this comment. ## Not a real heading -->",
  "## 変更内容",
  "## 受け入れ条件との対応",
].join("\n");

test("extracts H2 headings and ignores comments and code fences", () => {
  const fence = String.fromCharCode(96).repeat(3);
  const headings = extractH2Headings(
    [
      "## Why",
      "<!-- ## hidden -->",
      fence + "markdown",
      "## also hidden",
      fence,
      "### not an H2",
      "## 検証結果",
    ].join("\n"),
  );

  assert.deepEqual(headings, ["Why", "検証結果"]);
});

test("accepts a PR body that includes all template sections", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "## 変更内容",
      "Changed.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
});

test("reports missing template sections", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容"]);
});
