"use strict";

function withoutInlineCodeAndHtmlComments(markdown) {
  const source = String(markdown || "");
  const output = [];
  const backtick = String.fromCharCode(96);

  function maskRange(start, end) {
    let masked = "";
    for (let cursor = start; cursor < end; cursor += 1) {
      masked += source[cursor] === "\n" || source[cursor] === "\r"
        ? source[cursor]
        : " ";
    }
    output.push(masked);
  }

  function isEscapedBacktick(index) {
    let backslashCount = 0;
    for (let cursor = index - 1; cursor >= 0 && source[cursor] === "\\"; cursor -= 1) {
      backslashCount += 1;
    }
    return backslashCount % 2 === 1;
  }

  for (let index = 0; index < source.length;) {
    if (source[index] === backtick) {
      let openingEnd = index;
      while (source[openingEnd] === backtick) {
        openingEnd += 1;
      }

      if (isEscapedBacktick(index)) {
        output.push(source.slice(index, openingEnd));
        index = openingEnd;
        continue;
      }

      const delimiterLength = openingEnd - index;
      let search = openingEnd;
      let closingEnd = -1;

      while (search < source.length) {
        if (source[search] !== backtick) {
          search += 1;
          continue;
        }

        let runEnd = search;
        while (source[runEnd] === backtick) {
          runEnd += 1;
        }

        if (runEnd - search === delimiterLength) {
          closingEnd = runEnd;
          break;
        }

        search = runEnd;
      }

      if (closingEnd >= 0) {
        maskRange(index, closingEnd);
        index = closingEnd;
      } else {
        // An unmatched backtick run is literal Markdown, not an inline code span.
        output.push(source.slice(index, openingEnd));
        index = openingEnd;
      }
      continue;
    }

    if (source.startsWith("<!--", index)) {
      const commentClose = source.indexOf("-->", index + 4);
      const commentEnd = commentClose < 0 ? source.length : commentClose + 3;
      maskRange(index, commentEnd);
      index = commentEnd;
      continue;
    }

    output.push(source[index]);
    index += 1;
  }

  return output.join("");
}

const fs = require("node:fs");
const path = require("node:path");

function withoutCommentsAndCode(markdown) {
  // Handle block-level constructs first so fences inside raw HTML cannot end
  // that block, while fences outside HTML prevent their contents from being
  // interpreted as HTML.
  return withoutInlineCodeAndHtmlComments(withoutRawHtmlBlocks(String(markdown || "")));
}

function isCompleteHtmlTag(content) {
  const tagStart = content.match(/^<(\/?)([A-Za-z][A-Za-z0-9-]*)/);
  if (!tagStart) {
    return false;
  }

  let index = tagStart[0].length;
  if (tagStart[1] === "/") {
    return /^[ \t]*>[ \t]*$/.test(content.slice(index));
  }

  let quote = null;
  for (; index < content.length; index += 1) {
    const character = content[index];

    if (quote) {
      if (character === quote) {
        quote = null;
      }
      continue;
    }

    if (character === '"' || character === "'") {
      quote = character;
      continue;
    }

    if (character === "<") {
      return false;
    }

    if (character === ">") {
      return /^[ \t]*$/.test(content.slice(index + 1));
    }
  }

  return false;
}

function withoutRawHtmlBlocks(markdown) {
  const lines = String(markdown || "").split(/\r?\n/);
  const visibleLines = [];
  const grave = String.fromCharCode(96);
  let activeBlock = null;
  let activeFence = null;
  let paragraphOpen = false;
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
      if (line.trim() === "") {
        paragraphOpen = false;
      }
      // Fences inside a raw HTML block are literal HTML content, not Markdown.
      continue;
    }

    // Outside raw HTML, fenced code blocks take precedence over HTML parsing.
    if (activeFence) {
      const closing = line.match(new RegExp("^ {0,3}(" + grave + "+|~+)[ \\t]*$"));
      if (
        closing &&
        closing[1][0] === activeFence.character &&
        closing[1].length >= activeFence.length
      ) {
        activeFence = null;
      }
      visibleLines.push("");
      paragraphOpen = false;
      continue;
    }

    const opening = line.match(new RegExp("^ {0,3}(" + grave + "{3,}|~{3,})(.*)$"));
    if (opening) {
      const character = opening[1][0];
      const info = opening[2];
      // CommonMark disallows backticks in a backtick fence info string.
      if (!(character === grave && info.includes(grave))) {
        activeFence = { character, length: opening[1].length };
        visibleLines.push("");
        paragraphOpen = false;
        continue;
      }
    }

    // An indented code block starts outside an open paragraph with four
    // or more spaces (or a leading tab). It is not paragraph text, and its
    // contents must not keep the following HTML-block check in paragraph mode.
    if (!paragraphOpen && /^(?: {4,}|\t)/.test(line)) {
      visibleLines.push(line);
      paragraphOpen = false;
      continue;
    }

    const contentMatch = line.match(/^ {0,3}(.*)$/);
    if (!contentMatch || !contentMatch[1]) {
      visibleLines.push(line);
      paragraphOpen = false;
      continue;
    }

    const content = contentMatch[1];

    // ATX headings interrupt paragraphs.
    if (/^#{1,6}(?:[ \t]+|$)/.test(content)) {
      visibleLines.push(line);
      paragraphOpen = false;
      continue;
    }

    // Thematic breaks and setext underlines close the current paragraph too.
    if (
      /^(?:(?:\*[ \t]*){3,}|(?:-[ \t]*){3,}|(?:_[ \t]*){3,})$/.test(content) ||
      /^(?:=+[ \t]*|-{1,2}[ \t]*)$/.test(content)
    ) {
      visibleLines.push(line);
      paragraphOpen = false;
      continue;
    }

    // Block quote and list item markers start block containers and close
    // a paragraph at the current nesting level.
    if (/^>/.test(content) || /^(?:[-+*][ \t]+|[1-9][0-9]{0,8}[.)][ \t]+)/.test(content)) {
      visibleLines.push(line);
      paragraphOpen = false;
      continue;
    }

    const rawTag = content.match(/^<(script|pre|style|textarea)(?=[ \t/>])/i);
    if (rawTag) {
      // CommonMark type 1 ends at any of these close tags, not only the tag
      // corresponding to the opening element.
      const tagEnd = /<\/(?:pre|script|style|textarea)\s*>/i;
      if (!tagEnd.test(content)) {
        activeBlock = { endPattern: tagEnd };
      }
      paragraphOpen = false;
      continue;
    }

    const specialBlocks = [
      { start: /^<!--/, end: /-->/ },
      { start: /^<\?/, end: /\?>/ },
      { start: /^<!\[CDATA\[/, end: /\]\]>/ },
      { start: /^<![A-Z]/i, end: />/ },
    ];
    const special = specialBlocks.find((block) => block.start.test(content));
    if (special) {
      if (!special.end.test(content)) {
        activeBlock = { endPattern: special.end };
      }
      paragraphOpen = false;
      continue;
    }

    const blockTag = new RegExp("^(?:</?(?:" + blockTags + ")(?=[ \t/>]|$))", "i");
    if (blockTag.test(content)) {
      activeBlock = { untilBlank: true };
      paragraphOpen = false;
      continue;
    }

    // CommonMark raw HTML block type 7 cannot interrupt an open paragraph.
    // Recognize a complete tag only when no paragraph is currently open,
    // while still ignoring '>' characters inside quoted attribute values.
    if (isCompleteHtmlTag(content) && !paragraphOpen) {
      activeBlock = { untilBlank: true };
      paragraphOpen = false;
      continue;
    }

    visibleLines.push(line);
    paragraphOpen = true;
  }

  return visibleLines.join("\n");
}
function extractH2Headings(markdown) {
  const headings = [];

  for (const line of withoutCommentsAndCode(markdown).split(/\r?\n/)) {
    // CommonMark ATX headings allow up to three leading spaces and an optional
    // closing sequence of hash characters preceded by whitespace.
    const match = line.match(/^ {0,3}##[ \t]+(.+?)[ \t]*$/);
    if (!match) {
      continue;
    }

    const heading = match[1].replace(/[ \t]+#+[ \t]*$/, "").trim();
    if (heading) {
      headings.push(heading);
    }
  }

  return headings;
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
