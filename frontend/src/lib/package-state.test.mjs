import assert from "node:assert/strict";
import test from "node:test";
import {
  PackageStateRevision,
  PendingRequestCounter,
  withUnknownPackageState,
  withVerifiedActivePackageState,
} from "./package-state.ts";

test("updates only the verified package and does not claim configured provenance", () => {
  const existing = {
    chromium: {
      name: "chromium",
      configured: false,
      active: false,
    },
    vscode: {
      name: "vscode",
      configured: null,
      active: null,
    },
  };

  const next = withVerifiedActivePackageState(existing, "firefox");

  assert.deepEqual(next.firefox, {
    name: "firefox",
    configured: null,
    active: true,
  });
  assert.deepEqual(next.chromium, existing.chromium);
  assert.deepEqual(next.vscode, existing.vscode);
  assert.deepEqual(existing.firefox, undefined);
});

test("marks an unverified result unknown without claiming installation succeeded", () => {
  const existing = {
    firefox: { name: "firefox", configured: false, active: false },
    chromium: { name: "chromium", configured: true, active: true },
  };

  const next = withUnknownPackageState(existing, "firefox");

  assert.deepEqual(next.firefox, {
    name: "firefox",
    configured: null,
    active: null,
  });
  assert.deepEqual(next.chromium, existing.chromium);
  assert.deepEqual(existing.firefox, {
    name: "firefox",
    configured: false,
    active: false,
  });
});

test("keeps loading active until every overlapping request finishes", () => {
  const pending = new PendingRequestCounter();

  pending.begin();
  pending.begin();

  assert.equal(pending.finish(), true);
  assert.equal(pending.finish(), false);
});

test("newer overlapping requests supersede only the package IDs they share", () => {
  const revisions = new PackageStateRevision();
  const olderFirefox = revisions.advance("firefox");
  const olderChromium = revisions.advance("chromium");
  const newerFirefox = revisions.advance("firefox");

  assert.equal(revisions.isCurrent("firefox", olderFirefox), false);
  assert.equal(revisions.isCurrent("firefox", newerFirefox), true);
  assert.equal(revisions.isCurrent("chromium", olderChromium), true);
});

test("invalidates a package-state response that started before installation", () => {
  const revisions = new PackageStateRevision();
  const revisionBeforeInstall = revisions.current("firefox");

  revisions.advance("firefox");

  assert.equal(revisions.isCurrent("firefox", revisionBeforeInstall), false);
  assert.equal(revisions.isCurrent("firefox", revisions.current("firefox")), true);
  assert.equal(revisions.isCurrent("chromium", revisionBeforeInstall), true);
});
