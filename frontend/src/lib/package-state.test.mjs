import assert from "node:assert/strict";
import test from "node:test";
import { withInstalledPackageState } from "./package-state.ts";

test("marks a successfully installed package as configured and active", () => {
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

  const next = withInstalledPackageState(existing, "firefox");

  assert.deepEqual(next.firefox, {
    name: "firefox",
    configured: true,
    active: true,
  });
  assert.deepEqual(next.chromium, existing.chromium);
  assert.deepEqual(next.vscode, existing.vscode);
  assert.deepEqual(existing, {
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
  });
});

test("updates an existing package's state after a successful reinstall", () => {
  const next = withInstalledPackageState(
    {
      firefox: {
        name: "firefox",
        configured: true,
        active: false,
      },
    },
    "firefox",
  );

  assert.deepEqual(next.firefox, {
    name: "firefox",
    configured: true,
    active: true,
  });
});
