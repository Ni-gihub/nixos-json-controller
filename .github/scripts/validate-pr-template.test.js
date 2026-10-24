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


test("escaped backticks do not open or close inline code spans", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "A literal escaped backtick is written as \\` here.",
      "## 変更内容",
      "Changed.",
      "Use `code` in the example.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
});

test("backslashes inside code spans do not prevent closing delimiters", () => {
  const backslashClosedCodeSpan = "Example: `foo" + "\\" + "` is still a closed code span."; 
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      backslashClosedCodeSpan,
      "## 変更内容",
      "Changed.",
      "Use `code` in the example.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
});

test("unmatched backticks cannot mask headings across Markdown block boundaries", () => {
  const tick = String.fromCharCode(96);
  const bodies = [
    [
      "## Why",
      "Reason with an unmatched backtick " + tick,
      "",
      "## 変更内容",
      "Changed.",
      "",
      "Another unmatched backtick " + tick,
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
    [
      "## Why",
      "Reason with an unmatched backtick " + tick,
      "## 変更内容",
      "Changed.",
      "Another unmatched backtick " + tick,
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  ];

  for (const body of bodies) {
    const result = validatePullRequestBody(template, body);
    assert.equal(result.ok, true);
    assert.deepEqual(result.missingHeadings, []);
    assert.deepEqual(extractH2Headings(body), [
      "Why",
      "変更内容",
      "受け入れ条件との対応",
    ]);
  }
});

test("inline code containing HTML comment start markers does not hide subsequent headings", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Use the literal token `<!--` in the example; it is not an HTML comment.",
      "## 変更内容",
      "Changed.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
  assert.deepEqual(extractH2Headings("Text with `## not a heading` here\n## Why"), [
    "Why",
  ]);
});

test("inline HTML followed by a valid heading does not hide the heading", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "<span>説明</span>",
      "## 変更内容",
      "Changed.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
});

test("type 7 HTML tags do not start a raw block inside an open paragraph", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "<span>",
      "## 変更内容",
      "Changed.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
  assert.deepEqual(extractH2Headings("## Why\nReason.\n<span>\n## 変更内容"), [
    "Why",
    "変更内容",
  ]);
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

  const textareaResult = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "<textarea>",
      "",
      "## 変更内容",
      "</textarea>",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(textareaResult.ok, false);
  assert.deepEqual(textareaResult.missingHeadings, ["変更内容"]);

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


test("type 1 HTML block opening tags may end a line before the closing angle bracket", () => {
  for (const tag of ["pre", "script", "style", "textarea"]) {
    const body = [
      "## Why",
      "Reason.",
      "<" + tag,
      ">",
      "## 変更内容",
      "</" + tag + ">",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n");
    const result = validatePullRequestBody(template, body);

    assert.equal(result.ok, false, tag);
    assert.deepEqual(result.missingHeadings, ["変更内容"], tag);
    assert.deepEqual(extractH2Headings(body), [
      "Why",
      "受け入れ条件との対応",
    ], tag);
  }
});

test("self-closing type 1 tag names do not open raw HTML blocks", () => {
  for (const tag of ["pre", "script", "style", "textarea"]) {
    const body = [
      "## Why",
      "Reason.",
      "",
      "<" + tag + "/>",
      "## 変更内容",
      "Changed.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\\n");

    const result = validatePullRequestBody(template, body);

    assert.equal(result.ok, true, tag);
    assert.deepEqual(result.missingHeadings, [], tag);
    assert.deepEqual(extractH2Headings(body), [
      "Why",
      "変更内容",
      "受け入れ条件との対応",
    ], tag);
  }
});

test("fence-like lines inside raw HTML blocks do not expose fake headings", () => {
  const body = [
    "## Why",
    "Reason.",
    "<div>",
    "```markdown",
    "Example.",
    "```",
    "## 変更内容",
    "Still inside the raw HTML block.",
    "</div>",
    "",
    "## 受け入れ条件との対応",
    "Done.",
  ].join("\n");

  const result = validatePullRequestBody(template, body);

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容"]);
  assert.deepEqual(extractH2Headings(body), ["Why", "受け入れ条件との対応"]);
});

test("backticks inside raw HTML blocks cannot mask valid headings outside the block", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "",
      "<custom-tag>",
      "`",
      "</custom-tag>",
      "",
      "## 変更内容",
      "Changed.",
      "A literal unmatched backtick follows: `",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
  assert.deepEqual(
    extractH2Headings([
      "## Why",
      "Reason.",
      "",
      "<custom-tag>",
      "`",
      "</custom-tag>",
      "",
      "## 変更内容",
      "Changed.",
      "A literal unmatched backtick follows: `",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n")),
    ["Why", "変更内容", "受け入れ条件との対応"],
  );
});

test("raw HTML tags inside an HTML comment do not swallow later headings", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "",
      "<!--",
      "<pre>",
      "## 変更内容",
      "</pre>",
      "-->",
      "## 変更内容",
      "Changed.",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
});
test("raw HTML blocks with quoted greater-than attribute values cannot satisfy required sections", () => {
  for (const tag of ['<custom-tag title="a > b">', "<custom-tag title='a > b'>"]) {
    const result = validatePullRequestBody(
      template,
      [
        "## Why",
        "Reason.",
        "",
        tag,
        "## 変更内容",
        "Still inside the raw HTML block.",
        "",
        "## 受け入れ条件との対応",
        "Done.",
      ].join("\n"),
    );

    assert.equal(result.ok, false, tag);
    assert.deepEqual(result.missingHeadings, ["変更内容"], tag);
  }
});

test("indented code blocks close paragraph state before type 7 HTML blocks", () => {
  for (const codeLine of ["    example", "\texample"]) {
    const result = validatePullRequestBody(
      template,
      [
        "## Why",
        "Reason.",
        "",
        codeLine,
        "<custom-tag>",
        "## 変更内容",
        "Still inside the raw HTML block.",
        "",
        "## 受け入れ条件との対応",
        "Done.",
      ].join("\n"),
    );

    assert.equal(result.ok, false, JSON.stringify(codeLine));
    assert.deepEqual(result.missingHeadings, ["変更内容"], JSON.stringify(codeLine));
  }
});

test("block quotes and ordered list markers including leading zeros close paragraphs before type 7 HTML blocks", () => {
  for (const marker of [
    "> quote",
    "- list item",
    "1. ordered item",
    "01. ordered item",
    "001) ordered item",
    "000000001. ordered item",
  ]) {
    const result = validatePullRequestBody(
      template,
      [
        "## Why",
        "Reason.",
        marker,
        "<custom-tag>",
        "## 変更内容",
        "Still inside the raw HTML block.",
        "",
        "## 受け入れ条件との対応",
        "Done.",
      ].join("\n"),
    );

    assert.equal(result.ok, false, marker);
    assert.deepEqual(result.missingHeadings, ["変更内容"], marker);
  }
});

test("ordered list markers other than 1 do not interrupt an open paragraph", () => {
  const body = [
    "## Why",
    "Reason.",
    "2. this is still paragraph text",
    "<custom-tag>",
    "## 変更内容",
    "Changed.",
    "3) this is still paragraph text",
    "<custom-tag>",
    "## 受け入れ条件との対応",
    "Done.",
  ].join("\n");

  const result = validatePullRequestBody(template, body);

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
  assert.deepEqual(extractH2Headings(body), [
    "Why",
    "変更内容",
    "受け入れ条件との対応",
  ]);
});

test("thematic breaks close paragraphs before type 7 HTML blocks", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "***",
      "<custom-tag>",
      "## 変更内容",
      "Still inside the raw HTML block.",
      "",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容"]);
});

test("tab whitespace in ATX headings, thematic breaks, and setext underlines closes paragraphs", () => {
  const cases = [
    {
      name: "tab after ATX heading markers",
      separator: ["##\tSubheading"],
      expectedHeadings: ["Why", "Subheading", "受け入れ条件との対応"],
    },
    {
      name: "tabs between thematic break markers",
      separator: ["*\t*\t*"],
    },
    {
      name: "tab after setext underline",
      separator: ["Setext heading", "===\t"],
    },
  ];

  for (const item of cases) {
    const body = [
      "## Why",
      "Reason.",
      ...item.separator,
      "<custom-tag>",
      "## 変更内容",
      "Still inside the raw HTML block.",
      "",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n");
    const result = validatePullRequestBody(template, body);

    assert.equal(result.ok, false, item.name);
    assert.deepEqual(result.missingHeadings, ["変更内容"], item.name);
    assert.deepEqual(
      extractH2Headings(body),
      item.expectedHeadings || ["Why", "受け入れ条件との対応"],
      item.name,
    );
  }
});

test("fenced code blocks separate paragraphs before type 7 HTML blocks", () => {
  const result = validatePullRequestBody(
    template,
    [
      "## Why",
      "Reason.",
      "```text",
      "This is an example fence.",
      "```",
      "<custom-tag>",
      "## 変更内容",
      "Still inside the raw HTML block.",
      "",
      "## 受け入れ条件との対応",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容"]);
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


test("normalizes valid ATX H2 indentation and optional closing hashes", () => {
  const result = validatePullRequestBody(
    template,
    [
      "  ## Why",
      "Reason.",
      "## 変更内容 ##",
      "Changed.",
      "   ## 受け入れ条件との対応   ###",
      "Done.",
    ].join("\n"),
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
  assert.deepEqual(extractH2Headings("  ## Why\n## 変更内容 ##"), [
    "Why",
    "変更内容",
  ]);
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
