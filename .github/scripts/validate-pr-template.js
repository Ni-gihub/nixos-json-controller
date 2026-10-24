"use strict";

const fs = require("node:fs");
const path = require("node:path");

function withoutCommentsAndCode(markdown) {
  return String(markdown || "")
    .replace(/```[\\s\\S]*?```/g, "")
    .replace(/<!--[\\s\\S]*?-->/g, "");
}

function extractH2Headings(markdown) {
  return [...withoutCommentsAndCode(markdown).matchAll(/^##[ \\t]+(.+?)[ \\t]*$/gm)]
    .map((match) => match[1].trim())
    .filter(Boolean);
}

function validatePullRequestBody(template, body) {
  const requiredHeadings = extractH2Headings(template);
  const presentHeadings = new Set(extractH2Headings(body));
  const missingHeadings = requiredHeadings.filter((heading) => !presentHeadings.has(heading));

  return {
    ok: requiredHeadings.length > 0 && missingHeadings.length === 0,
    requiredHeadings,
    missingHeadings,
  };
}

function validateEvent(eventPath, templatePath) {
  if (!eventPath) {
    throw new Error("Usage: node validate-pr-template.js <github-event-json-path>");
  }

  const event = JSON.parse(fs.readFileSync(eventPath, "utf8"));
  const body = event.pull_request?.body;
  if (typeof body !== "string") {
    throw new Error("The GitHub event does not contain a pull request body.");
  }

  const template = fs.readFileSync(
    templatePath || path.join(__dirname, "..", "pull_request_template.md"),
    "utf8",
  );
  const result = validatePullRequestBody(template, body);

  if (!result.requiredHeadings.length) {
    throw new Error("No H2 headings were found in the pull request template.");
  }

  if (!result.ok) {
    process.stderr.write(
      [
        "PR body is missing sections from .github/pull_request_template.md:",
        ...result.missingHeadings.map((heading) => `- ## ${heading}`),
        "",
        "Fill every template section. Use '該当なし' when a section does not apply.",
      ].join("\\n") + "\\n",
    );
    return false;
  }

  process.stdout.write(
    `PR body contains all ${result.requiredHeadings.length} required template sections.\\n`,
  );
  return true;
}

if (require.main === module) {
  try {
    process.exitCode = validateEvent(
      process.argv[2],
      process.argv[3],
    ) ? 0 : 1;
  } catch (error) {
    process.stderr.write(`${error.message}\\n`);
    process.exitCode = 1;
  }
}

module.exports = { extractH2Headings, validatePullRequestBody, validateEvent };
