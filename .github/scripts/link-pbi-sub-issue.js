"use strict";

const CHILD_TYPES = new Set(["Task", "Bug", "Spike"]);
const TRUSTED_ASSOCIATIONS = new Set(["OWNER", "MEMBER", "COLLABORATOR"]);

function extractParentPbiValue(body) {
  const match = String(body || "").match(
    /^### Parent PBI number \\(optional\\)[ \\t]*\\r?\\n([^\\r\\n]*)/m,
  );
  return match ? match[1].trim() : null;
}

function parseIssueNumber(value) {
  if (typeof value !== "string" || !/^[1-9]\\d*$/.test(value)) {
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

function requestedParentNumber(issue, core) {
  const rawParent = extractParentPbiValue(issue.body);
  if (!rawParent) {
    return { parentNumber: null, reason: "no-parent-requested" };
  }

  const parentNumber = parseIssueNumber(rawParent);
  if (!parentNumber) {
    core.warning(
      'Parent PBI number must contain only a positive issue number (for example, "177"). No parent was linked.',
    );
    return { parentNumber: null, reason: "invalid-parent-number" };
  }

  if (parentNumber === issue.number) {
    core.warning("An issue cannot be its own parent. No parent was linked.");
    return { parentNumber: null, reason: "self-parent" };
  }

  return { parentNumber, reason: null };
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
 * Always read the current Issue body from GitHub instead of trusting an event
 * snapshot. Re-read it immediately before the write so an edit during this run
 * cannot knowingly attach the Issue to an obsolete PBI. Existing parent links
 * are never silently replaced.
 */
async function linkIssueToPbi({ github, context, core }) {
  const eventIssue = context.payload?.issue;
  if (!eventIssue || eventIssue.pull_request) {
    return { action: "skip", reason: "not-an-issue" };
  }

  if (!TRUSTED_ASSOCIATIONS.has(eventIssue.author_association)) {
    core.warning(
      "Automatic parent linking is limited to repository owners, members, and collaborators. A maintainer can add this Issue from the parent's Sub-issues menu.",
    );
    return { action: "skip", reason: "untrusted-author" };
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

  const initialRequest = requestedParentNumber(child, core);
  if (!initialRequest.parentNumber) {
    if (initialRequest.reason === "no-parent-requested") {
      core.info("No Parent PBI number is currently provided; leaving the issue standalone.");
    }
    return { action: "skip", reason: initialRequest.reason };
  }
  const parentNumber = initialRequest.parentNumber;

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

  // An edit may have changed or cleared the Parent PBI number while the
  // parent Issue was being fetched. Re-read the child before linking.
  const { data: latestChild } = await github.rest.issues.get({
    owner,
    repo,
    issue_number: child.number,
  });
  if (latestChild.pull_request || !CHILD_TYPES.has(currentIssueType(latestChild))) {
    core.warning("The child Issue Type changed during processing; no parent was linked.");
    return { action: "skip", reason: "child-type-changed" };
  }

  const latestRequest = requestedParentNumber(latestChild, core);
  if (!latestRequest.parentNumber) {
    if (latestRequest.reason === "no-parent-requested") {
      core.info("Parent PBI number was cleared during processing; no parent was linked.");
    }
    return { action: "skip", reason: latestRequest.reason };
  }
  if (latestRequest.parentNumber !== parentNumber) {
    core.info(
      `Parent PBI number changed from #${parentNumber} to #${latestRequest.parentNumber} during processing; a newer edited event can handle the current value.`,
    );
    return { action: "skip", reason: "parent-request-changed" };
  }

  // Check current hierarchy as late as possible and never replace an existing
  // parent. The workflow also cancels superseded runs for this Issue.
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
      sub_issue_id: latestChild.id,
      headers: { "X-GitHub-Api-Version": "2022-11-28" },
    },
  );

  core.info(`Linked #${latestChild.number} as a sub-issue of PBI #${parentNumber}.`);
  return { action: "linked", parentNumber, childNumber: latestChild.number };
}

module.exports = {
  extractParentPbiValue,
  parseIssueNumber,
  currentIssueType,
  requestedParentNumber,
  linkIssueToPbi,
};
