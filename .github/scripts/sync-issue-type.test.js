"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const { inferIssueType, planIssueType } = require("./sync-issue-type.js");

test("uses explicit type labels for a PBI", () => {
  assert.deepEqual(
    inferIssueType({
      title: "[PBI] Install the GUI from a Nix profile",
      labels: [{ name: "PBI" }, { name: "gui" }],
    }),
    {
      type: "PBI",
      source: "label:pbi+title-prefix:[PBI]",
      reason: null,
    },
  );
});

test("infers Task from an Issue Form title prefix", () => {
  assert.deepEqual(
    inferIssueType({
      title: "[TASK] Audit Tauri permissions",
      labels: [{ name: "security" }],
    }),
    {
      type: "Task",
      source: "title-prefix:[TASK]",
      reason: null,
    },
  );
});

test("keeps Spike investigations as Tasks while retaining the SPIKE label", () => {
  assert.equal(
    inferIssueType({
      title: "[SPIKE] Investigate icon instability",
      labels: [{ name: "bug" }, { name: "SPIKE" }, { name: "gui" }],
    }).type,
    "Task",
  );
});

test("falls back to Bug for legacy issues with only the bug label", () => {
  assert.equal(
    inferIssueType({
      title: "Icon is blank after uninstall",
      labels: [{ name: "bug" }, { name: "gui" }],
    }).type,
    "Bug",
  );
});

test("does not guess when type-label and title-prefix evidence conflict", () => {
  assert.deepEqual(
    inferIssueType({
      title: "[TASK] Review the release plan",
      labels: [{ name: "PBI" }],
    }),
    {
      type: null,
      source: null,
      reason: "conflicting-evidence:PBI,Task",
    },
  );
});

test("preserves an already assigned Issue Type", () => {
  assert.deepEqual(
    planIssueType({
      number: 1,
      title: "[TASK] Already classified",
      type: { name: "PBI" },
      labels: [{ name: "TASK" }],
    }),
    {
      action: "skip",
      type: "PBI",
      source: null,
      reason: "already-typed",
    },
  );
});

test("does not classify pull requests", () => {
  assert.equal(
    planIssueType({
      number: 2,
      title: "[TASK] This is a pull request",
      pull_request: { url: "https://example.invalid/pull/2" },
      labels: [{ name: "TASK" }],
    }).reason,
    "pull-request",
  );
});

test("leaves unclassified Issues unchanged", () => {
  assert.deepEqual(
    planIssueType({
      number: 3,
      title: "A discussion without a type signal",
      labels: [{ name: "question" }],
    }),
    {
      action: "skip",
      type: null,
      source: null,
      reason: "no-recognized-type-signal",
    },
  );
});
