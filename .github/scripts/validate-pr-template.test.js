"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const { extractH2Headings, validatePullRequestBody } = require("./validate-pr-template.js");

const template = `## Why
<!-- Remove this comment. ## Not a real heading -->
## 変更内容
## 受け入れ条件との対応
`;

test("extracts H2 headings from the template and ignores comments and code fences", () => {
  const headings = extractH2Headings(
    `## Why
<!-- ## hidden -->
```markdown
## also hidden
```
### not an H2
## 検証結果
`,
  );

  assert.deepEqual(headings, ["Why", "検証結果"]);
});

test("accepts a PR body that includes all template sections", () => {
  const result = validatePullRequestBody(
    template,
    `## Why
Reason.
## 変更内容
Changed.
## 受け入れ条件との対応
Done.
`,
  );

  assert.equal(result.ok, true);
  assert.deepEqual(result.missingHeadings, []);
});

test("reports missing template sections", () => {
  const result = validatePullRequestBody(
    template,
    `## Why
Reason.
## 受け入れ条件との対応
Done.
`,
  );

  assert.equal(result.ok, false);
  assert.deepEqual(result.missingHeadings, ["変更内容"]);
});
