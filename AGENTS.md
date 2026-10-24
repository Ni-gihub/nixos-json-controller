# Repository instructions for AI-assisted changes

## Before creating or editing an Issue

1. Read `docs/project-management/issue-writing.md` and `docs/project-management/backlog-management.md`.
2. Read the relevant `.github/ISSUE_TEMPLATE/*.yml` form before drafting the Issue. Follow the form's fields and Type; do not rely only on the contact-link or guide URL.
3. Search existing open and closed Issues/PBIs before creating a new one. Reuse and link the existing PBI when the work contributes to its outcome.
4. **Default relationship:** a Task, Bug, or Spike that contributes to a user-facing or project outcome should be a native GitHub sub-issue of its PBI. For a new piece of work, identify the PBI first; if no suitable PBI exists, create the PBI first when the work represents a distinct user value, then create the Task/Bug/Spike with the PBI number in `Parent PBI number (optional)`.
5. Leave a Task/Bug/Spike independent only when it genuinely cuts across PBIs or has no single appropriate PBI (for example, a repository-wide housekeeping issue). State why it is independent in the Issue body and link related PBIs. Do not use a blank parent field merely because the link was inconvenient.
6. After creating a child Issue, fetch it and verify that the intended native Sub-issues relationship was established. The parent number in the body is an automation request, not proof that the relationship exists. If the author is not eligible for automatic linking, ask a maintainer to add it from the parent's Sub-issues menu.

PBI Issues represent user value and are not themselves children of another PBI. Project #166 is the project charter and PBL #167 is the backlog index; neither is a substitute for a PBI parent.

## Before creating or editing a pull request

1. Read the current base-branch version of `.github/pull_request_template.md` before drafting or updating the PR body. Do this every time; do not assume a previous PR or remembered template is current.
2. Use every H2 section in the template and complete it with accurate information. When a section is not applicable, write `該当なし` and briefly explain why instead of silently deleting the section. Do not leave template placeholders.
3. Link the directly addressed Task/Bug/Spike using `Closes #N` only when merging the PR completes that entire Issue; use `Refs #N` for partial work or related investigations. Link the PBI with `Refs #N`, using `Closes` only when the PBI's full acceptance criteria are satisfied by this PR.
4. Report tests and CI only after checking their actual results for the current head commit. Explicitly state checks that were not run. Keep risks and unresolved questions visible.
5. GitHub's pull request template is not automatically applied when creating a PR through the REST API or another direct integration. Include the complete, template-compliant body in the creation/update request.
6. After creating or updating the PR, fetch the PR again and verify the saved body contains every heading from the current base-branch template and that the Issue/PBI links are correct.

The dedicated PR-template validation workflow checks that PR bodies contain each H2 section from the base-branch template. This automated check complements, but does not replace, reading the template and completing its sections.

## When reviewing a pull request

1. Read `.github/AI_REVIEW_GUIDELINES.md` before reviewing. Inspect the full PR diff, relevant code and Issues, tests, and CI results for the current head commit.
2. For an actionable finding that can be tied to a changed diff line, post an inline review comment on that line. A top-level review summary alone is not enough for line-specific code findings. Use the review body for the overall assessment, CI/test results, and findings that cannot be reliably anchored to a changed line.
3. When responding to review feedback, inspect all existing review threads, make the required changes, verify the resulting diff and tests/CI, and reply in the relevant threads. Resolve a thread only after verifying that the reported issue has actually been addressed.
