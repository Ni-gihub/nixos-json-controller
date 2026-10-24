"use strict";

const fs = require("node:fs");
const path = require("node:path");

function withoutCommentsAndCode(markdown) {
  const lines = String(markdown || "").split(/\r?\n/);
  const visibleLines = [];
  const grave = String.fromCharCode(96);
  let activeFence = null;

  for (const line of lines) {
    if (activeFence) {
      const closing = line.match(new RegExp("^ {0,3}(" + grave + "+|~+)[ \\t]*$"));
      if (
        closing &&
        closing[1][0] === activeFence.character &&
        closing[1].length >= activeFence.length
      ) {
        activeFence = null;
      }
      continue;
    }

    const opening = line.match(new RegExp("^ {0,3}(" + grave + "{3,}|~{3,})(.*)$"));
    if (opening) {
      const character = opening[1][0];
      const length = opening[1].length;
      const info = opening[2];
      // CommonMark does not allow backticks in a backtick fence info string.
      if (!(character === grave && info.includes(grave))) {
        activeFence = { character, length };
        continue;
      }
    }

    visibleLines.push(line);
  }

  return withoutRawHtmlBlocks(
    visibleLines.join("\n").replace(/<!--[\\s\\S]*?(?:-->|$)/g, ""),
  );
}

function withoutRawHtmlBlocks(markdown) {
  const lines = String(markdown || "").split(/\r?\n/);
  const visibleLines = [];
  let activeBlock = null;
  const blockTags =
    "address|article|aside|base|basefont|blockquote|body|caption|center|col|colgroup|" +
    "dd|details|dialog|dir|div|dl|dt|fieldset|figcaption|figure|footer|form|frame|" +
    "frameset|h1|h2|h3|h4|h5|h6|head|header|hr|html|iframe|legend|li|link|main|" +
    "menu|menuitem|meta|nav|noframes|ol|optgroup|option|p|param|search|section|" +
    "summary|table|tbody|td|tfoot|th|thead|title|tr|track|ul";

  for (const line of lines) {
    if (activeBlock) {
      const closes = activeBlock.endPattern && activeBlock.endPattern.test(line);
      const endsAtBlank = activeBlock.untilBlank && line.trim() === "";
      if (closes) {
        activeBlock = null;
      } else if (endsAtBlank) {
        activeBlock = null;
        visibleLines.push(line);
      }
      continue;
    }

    const contentMatch = line.match(/^ {0,3}(.*)$/);
    if (!contentMatch || !contentMatch[1]) {
      visibleLines.push(line);
      continue;
    }

    const content = contentMatch[1];
    const rawTag = content.match(/^<(script|pre|style)(?=[ \t/>])/i);
    if (rawTag) {
      const tagEnd = new RegExp("</" + rawTag[1] + "\\s*>", "i");
      if (!tagEnd.test(content)) {
        activeBlock = { endPattern: tagEnd };
      }
      continue;
    }

    const specialBlocks = [
      { start: /^<\?/, end: /\?>/ },
      { start: /^<!\[CDATA\[/, end: /\]\]>/ },
      { start: /^<![A-Z]/i, end: />/ },
    ];
    const special = specialBlocks.find((block) => block.start.test(content));
    if (special) {
      if (!special.end.test(content)) {
        activeBlock = { endPattern: special.end };
      }
      continue;
    }

    const blockTag = new RegExp("^(?:</?(?:" + blockTags + ")(?=[ \t/>]|$))", "i");
    if (blockTag.test(content)) {
      activeBlock = { untilBlank: true };
      continue;
    }

    // CommonMark raw HTML block type 7: a complete open or closing tag at line start.
    const completeTag = /^<\/?[A-Za-z][A-Za-z0-9-]*(?:[ \t]+[^<>]*?)?[ \t]*\/?>/;
    if (completeTag.test(content)) {
      activeBlock = { untilBlank: true };
      continue;
    }

    visibleLines.push(line);
  }

  return visibleLines.join("\n");
}
function extractH2Headings(markdown) {
  return [...withoutCommentsAndCode(markdown).matchAll(/^##[ \t]+(.+?)[ \t]*$/gm)]
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
  const body = event.pull_request && event.pull_request.body;
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
        ...result.missingHeadings.map((heading) => "- ## " + heading),
        "",
        "Fill every template section. Use '該当なし' when a section does not apply.",
      ].join("\n") + "\n",
    );
    return false;
  }

  process.stdout.write(
    "PR body contains all " + result.requiredHeadings.length + " required template sections.\n",
  );
  return true;
}

if (require.main === module) {
  try {
    process.exitCode = validateEvent(process.argv[2], process.argv[3]) ? 0 : 1;
  } catch (error) {
    process.stderr.write(error.message + "\n");
    process.exitCode = 1;
  }
}

module.exports = { extractH2Headings, validatePullRequestBody, validateEvent };
