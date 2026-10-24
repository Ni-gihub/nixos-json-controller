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

test("extracts H2 headings and ignores HTML comments and both fence styles", () => {
  const backticks = String.fromCharCode(96).repeat(3);
  const tildes = "~~~";
  const headings = extractH2Headings(
    [
      "## Why",
      "<!-- ## hidden comment -->",
      backticks + "markdown",
      "## hidden in backtick fence",
      backticks,
      tildes + "markdown",
      "## hidden in tilde fence",
      "~~~~",
      "### not an H2",
      "## 検証結果",
    ].join("\n"),
  );

  assert.deepEqual(headings, ["Why", "検証結果"]);
});

test("headings in multiline HTML comments cannot satisfy required PR sections", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "<!--",
      "## 変更内容",
      "-->",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容"]);
});

test("raw HTML blocks cannot satisfy required PR sections", () => {
  const preResult = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "<pre>",
      "## 変更内容",
      "</pre>",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(preResult.ok, false);
  assert.deepEqual(preResult.missingHeadings, ["変更内容"]);

  const divResult = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "<div>",
      "## 変更内容",
      "</div>",
      "",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(divResult.ok, false);
  assert.deepEqual(divResult.missingHeadings, ["変更内容"]);
});

test("unclosed HTML comments cannot satisfy required PR sections", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "<!-- unclosed comment",
      "## 変更内容",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容", "受け入れ条件との対応"]);
});

test("tilde-fenced headings cannot satisfy required PR sections", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "~~~markdown",
      "## 変更内容",
      "~~~",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容"]);
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
