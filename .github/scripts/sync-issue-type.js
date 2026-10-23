/**
 * Infer and apply GitHub Issue Types for NXC issues.
 *
 * This script intentionally only assigns a type when an issue has no type.
 * Existing manual choices are preserved. Ambiguous classification is skipped.
 */
"use strict";

const TYPE_LABELS = Object.freeze({
  project: "Project",
  pbl: "PBL",
  pbi: "PBI",
  task: "Task",
  // Spike is a work subtype in this repository, represented by Task + the SPIKE label.
  spike: "Task",
});

const TITLE_PREFIX_TYPES = Object.freeze({
  PROJECT: "Project",
  PBL: "PBL",
  PBI: "PBI",
  TASK: "Task",
  BUG: "Bug",
  SPIKE: "Task",
});

function issueLabels(issue) {
  return (issue.labels || [])
    .map((label) => (typeof label === "string" ? label : label?.name))
    .filter((label) => typeof label === "string")
    .map((label) => label.trim().toLowerCase());
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

/**
 * Return the proposed type and the evidence used to infer it.
 * When title and label evidence disagree, avoid guessing.
 */
function inferIssueType(issue) {
  const labels = issueLabels(issue);
  const evidence = [];

  for (const label of labels) {
    const type = TYPE_LABELS[label];
    if (type) evidence.push({ type, source: `label:${label}` });
  }

  const title = typeof issue?.title === "string" ? issue.title : "";
  const titleMatch = title.match(/^\[(PROJECT|PBL|PBI|TASK|BUG|SPIKE)\](?:\s|$)/i);
  if (titleMatch) {
    const key = titleMatch[1].toUpperCase();
    evidence.push({ type: TITLE_PREFIX_TYPES[key], source: `title-prefix:[${key}]` });
  }

  const uniqueTypes = [...new Set(evidence.map((item) => item.type))];
  if (uniqueTypes.length > 1) {
    return {
      type: null,
      source: null,
      reason: `conflicting-evidence:${uniqueTypes.join(",")}`,
    };
  }

  if (uniqueTypes.length === 1) {
    return {
      type: uniqueTypes[0],
      source: [...new Set(evidence.map((item) => item.source))].join("+"),
      reason: null,
    };
  }

  // The generic bug label is only a fallback: it often coexists with a
  // more specific type such as Spike, which should be inferred from its
  // explicit type label or title prefix above.
  if (labels.includes("bug")) {
    return { type: "Bug", source: "label:bug", reason: null };
  }

  return { type: null, source: null, reason: "no-recognized-type-signal" };
}

function planIssueType(issue) {
  if (issue?.pull_request) {
    return { action: "skip", type: null, source: null, reason: "pull-request" };
  }

  const existingType = currentIssueType(issue);
  if (existingType) {
    return {
      action: "skip",
      type: existingType,
      source: null,
      reason: "already-typed",
    };
  }

  const inferred = inferIssueType(issue);
  if (!inferred.type) {
    return { action: "skip", ...inferred };
  }

  return { action: "set", ...inferred };
}

async function applyPlan({ github, owner, repo, issue, plan, core }) {
  const response = await github.rest.issues.update({
    owner,
    repo,
    issue_number: issue.number,
    type: plan.type,
  });

  let updated = response.data;
  let actualType = currentIssueType(updated);

  // GitHub can silently ignore a type update when the token lacks the
  // required repository write access. Verify the persisted value explicitly.
  if (!actualType || actualType.toLowerCase() !== plan.type.toLowerCase()) {
    const reread = await github.rest.issues.get({
      owner,
      repo,
      issue_number: issue.number,
    });
    updated = reread.data;
    actualType = currentIssueType(updated);
  }

  if (!actualType || actualType.toLowerCase() !== plan.type.toLowerCase()) {
    throw new Error(
      `GitHub did not persist the type for #${issue.number}; requested "${plan.type}", received "${actualType || "none"}". Check workflow permissions and the organization's Issue Types.`,
    );
  }

  core.info(`#${issue.number} ${issue.title}: set type to ${actualType} (${plan.source})`);
  return {
    number: issue.number,
    title: issue.title,
    currentType: currentIssueType(issue) || "",
    proposedType: plan.type,
    source: plan.source,
    result: "updated",
    reason: "",
  };
}

async function processIssue({ github, owner, repo, issue, apply, core }) {
  const plan = planIssueType(issue);

  if (plan.action === "skip") {
    return {
      number: issue.number,
      title: issue.title,
      currentType: currentIssueType(issue) || "",
      proposedType: "",
      source: "",
      result: "skipped",
      reason: plan.reason,
    };
  }

  if (!apply) {
    core.info(`DRY RUN #${issue.number} ${issue.title}: would set ${plan.type} (${plan.source})`);
    return {
      number: issue.number,
      title: issue.title,
      currentType: "",
      proposedType: plan.type,
      source: plan.source,
      result: "preview",
      reason: "",
    };
  }

  return applyPlan({ github, owner, repo, issue, plan, core });
}

async function writeSummary(core, heading, results) {
  const rows = [
    ["Issue", "Title", "Current type", "Proposed type", "Result", "Reason / evidence"],
    ...results.map((item) => [
      `#${item.number}`,
      item.title || "",
      item.currentType || "",
      item.proposedType || "",
      item.result || "",
      item.reason || item.source || "",
    ]),
  ];
  await core.summary.addHeading(heading).addTable(rows).write();
}

async function syncTriggeredIssue({ github, context, core }) {
  const owner = context.repo.owner;
  const repo = context.repo.repo;
  const issueNumber = context.payload.issue?.number;

  if (!issueNumber || context.payload.issue?.pull_request) {
    core.info("No standard issue to classify; skipping.");
    return;
  }

  // Fetch the current record to avoid acting on stale event payload data.
  const response = await github.rest.issues.get({
    owner,
    repo,
    issue_number: issueNumber,
  });
  const result = await processIssue({
    github,
    owner,
    repo,
    issue: response.data,
    apply: true,
    core,
  });

  await writeSummary(core, "Issue Type synchronization", [result]);
}

async function syncBacklog({ github, context, core, apply }) {
  const owner = context.repo.owner;
  const repo = context.repo.repo;
  const issues = await github.paginate(github.rest.issues.listForRepo, {
    owner,
    repo,
    state: "all",
    per_page: 100,
  });

  const results = [];
  let failures = 0;
  for (const issue of issues) {
    if (issue.pull_request) continue;

    try {
      results.push(
        await processIssue({ github, owner, repo, issue, apply, core }),
      );
    } catch (error) {
      failures += 1;
      const message = error instanceof Error ? error.message : String(error);
      core.error(`#${issue.number} ${issue.title}: ${message}`);
      results.push({
        number: issue.number,
        title: issue.title,
        currentType: currentIssueType(issue) || "",
        proposedType: "",
        source: "",
        result: "failed",
        reason: message,
      });
    }
  }

  await writeSummary(
    core,
    apply ? "Issue Type backfill (applied)" : "Issue Type backfill (preview only)",
    results,
  );
  core.info(
    `Checked ${results.length} issues: ${results.filter((item) => item.result === "updated").length} updated, ${results.filter((item) => item.result === "preview").length} proposed, ${results.filter((item) => item.result === "skipped").length} skipped, ${failures} failed.`,
  );

  if (failures > 0) {
    core.setFailed(`${failures} Issue Type update(s) failed; inspect the job log and summary.`);
  }
}

module.exports = {
  currentIssueType,
  inferIssueType,
  planIssueType,
  processIssue,
  syncBacklog,
  syncTriggeredIssue,
};
