"use strict";

const CHILD_TYPES = new Set(["Task", "Bug", "Spike"]);

function extractParentPbiValue(body) {
  const match = String(body || "").match(
    /^### Parent PBI number \(optional\)[ \t]*\r?\n([^\r\n]*)/m,
  );
  return match ? match[1].trim() : null;
}

function parseIssueNumber(value) {
  if (typeof value !== "string" || !/^[1-9]\d*$/.test(value)) {
    return null;
  }
  const number = Number(value);
  return Number.isSafeInteger(number) ? number : null;
}

function currentIssueType(issue) {
  if (typeof issue?.type === "string") {
    return issue.type.trim() || null;
  }
  if (issue?.type && typeof issue.type.name === "string") {
    return issue.type.name.trim() || null;
  }
  return null;
}

async function getExistingParent({ github, owner, repo, issueNumber }) {
  try {
    const response = await github.request(
      "GET /repos/{owner}/{repo}/issues/{issue_number}/parent",
      { owner, repo, issue_number: issueNumber },
    );
    return response.data;
  } catch (error) {
    if (error.status === 404) {
      return null;
    }
    throw error;
  }
}

/**
 * Link a Task, Bug, or Spike to a PBI through GitHub's native Sub-issues API.
 * The optional form field is only a request; the issue Types and current parent
 * are verified before any write. Existing parent relationships are never
 * silently replaced.
 */
async function linkIssueToPbi({ github, context, core }) {
  const eventIssue = context.payload?.issue;
  if (!eventIssue || eventIssue.pull_request) {
    return { action: "skip", reason: "not-an-issue" };
  }

  const trustedAssociations = new Set(["OWNER", "MEMBER", "COLLABORATOR"]);
  if (!trustedAssociations.has(eventIssue.author_association)) {
    core.warning(
      "Automatic parent linking is limited to repository owners, members, and collaborators. A maintainer can add this Issue from the parent's Sub-issues menu.",
    );
    return { action: "skip", reason: "untrusted-author" };
  }

  const rawParent = extractParentPbiValue(eventIssue.body);
  if (!rawParent) {
    core.info("No Parent PBI number provided; leaving the issue standalone.");
    return { action: "skip", reason: "no-parent-requested" };
  }

  const parentNumber = parseIssueNumber(rawParent);
  if (!parentNumber) {
    core.warning(
      'Parent PBI number must contain only a positive issue number (for example, "177"). No parent was linked.',
    );
    return { action: "skip", reason: "invalid-parent-number" };
  }

  if (parentNumber === eventIssue.number) {
    core.warning("An issue cannot be its own parent. No parent was linked.");
    return { action: "skip", reason: "self-parent" };
  }

  const { owner, repo } = context.repo;
  const { data: child } = await github.rest.issues.get({
    owner,
    repo,
    issue_number: eventIssue.number,
  });

  if (child.pull_request) {
    return { action: "skip", reason: "pull-request" };
  }

  const childType = currentIssueType(child);
  if (!CHILD_TYPES.has(childType)) {
    core.warning(
      `Only Task, Bug, or Spike Issues can be linked automatically as PBI sub-issues; #${child.number} has Type "${childType || "unset"}".`,
    );
    return { action: "skip", reason: "unsupported-child-type" };
  }

  let parent;
  try {
    ({ data: parent } = await github.rest.issues.get({
      owner,
      repo,
      issue_number: parentNumber,
    }));
  } catch (error) {
    if (error.status === 404) {
      core.warning(`Parent Issue #${parentNumber} was not found. No parent was linked.`);
      return { action: "skip", reason: "parent-not-found" };
    }
    throw error;
  }

  if (parent.pull_request) {
    core.warning(`#${parentNumber} is a pull request, not a PBI Issue.`);
    return { action: "skip", reason: "parent-is-pull-request" };
  }

  const parentType = currentIssueType(parent);
  if (parentType !== "PBI") {
    core.warning(
      `Parent Issue #${parentNumber} must have Type "PBI"; it has Type "${parentType || "unset"}". No parent was linked.`,
    );
    return { action: "skip", reason: "parent-is-not-pbi" };
  }

  const existingParent = await getExistingParent({
    github,
    owner,
    repo,
    issueNumber: child.number,
  });

  if (existingParent?.number === parentNumber) {
    core.info(`Issue #${child.number} is already a sub-issue of PBI #${parentNumber}.`);
    return { action: "skip", reason: "already-linked" };
  }

  if (existingParent) {
    core.warning(
      `Issue #${child.number} already has parent #${existingParent.number}; refusing to replace it with #${parentNumber}.`,
    );
    return { action: "skip", reason: "different-parent-exists" };
  }

  await github.request(
    "POST /repos/{owner}/{repo}/issues/{issue_number}/sub_issues",
    {
      owner,
      repo,
      issue_number: parentNumber,
      sub_issue_id: child.id,
      headers: { "X-GitHub-Api-Version": "2022-11-28" },
    },
  );

  core.info(`Linked #${child.number} as a sub-issue of PBI #${parentNumber}.`);
  return { action: "linked", parentNumber, childNumber: child.number };
}

module.exports = {
  extractParentPbiValue,
  parseIssueNumber,
  currentIssueType,
  linkIssueToPbi,
};
