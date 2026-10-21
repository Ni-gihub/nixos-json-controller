import assert from "node:assert/strict";
import test from "node:test";
import { submitInstallPassword } from "./install-flow.ts";

test("closes and clears the password prompt before a pending install settles", async () => {
  const calls = [];
  let finishInstall;
  const pendingInstall = new Promise((resolve) => {
    finishInstall = resolve;
  });

  const submission = submitInstallPassword("test-password", {
    closePasswordPrompt: () => calls.push("close"),
    clearPassword: () => calls.push("clear"),
    install: (password) => {
      calls.push("install");
      assert.equal(password, "test-password");
      return pendingInstall;
    },
  });

  assert.deepEqual(calls, ["close", "clear", "install"]);
  let settled = false;
  void submission.then(() => {
    settled = true;
  });

  await Promise.resolve();
  assert.equal(settled, false);

  finishInstall();
  await submission;
  assert.equal(settled, true);
});

test("clears the prompt before a failed installation and propagates its failure", async () => {
  const calls = [];

  await assert.rejects(
    submitInstallPassword("wrong-password", {
      closePasswordPrompt: () => calls.push("close"),
      clearPassword: () => calls.push("clear"),
      install: async () => {
        calls.push("install");
        throw new Error("authentication failed");
      },
    }),
    /authentication failed/,
  );

  assert.deepEqual(calls, ["close", "clear", "install"]);
});

import { renderToStaticMarkup } from "react-dom/server";
import React from "react";
import { createServer } from "vite";

test("renders a compact visible progress label with an accessible full label", async () => {
  const vite = await createServer({
    configFile: "./vite.config.ts",
    logLevel: "silent",
    server: { middlewareMode: true },
    appType: "custom",
  });

  try {
    const { default: InstallProgressIndicator } = await vite.ssrLoadModule(
      "/src/components/InstallProgressIndicator.tsx",
    );
    const markup = renderToStaticMarkup(
      React.createElement(InstallProgressIndicator, {
        label: "処理中...",
        ariaLabel: "インストール中...",
      }),
    );

    assert.match(markup, /role="status"/);
    assert.match(markup, /aria-live="polite"/);
    assert.match(markup, /aria-label="インストール中\.\.\."/);
    assert.match(markup, /処理中\.\.\./);
    assert.match(markup, /<svg/);
    assert.match(markup, /<circle/);
  } finally {
    await vite.close();
  }
});


test("creates a fresh install-button instance when the package changes", async () => {
  const vite = await createServer({
    configFile: "./vite.config.ts",
    logLevel: "silent",
    server: { middlewareMode: true },
    appType: "custom",
  });

  try {
    const { default: InstallButton } = await vite.ssrLoadModule(
      "/src/components/InstallButton.tsx",
    );
    const oldPackageElement = InstallButton({
      packageId: "firefox",
      appName: "Firefox",
      installed: false,
    });
    const newPackageElement = InstallButton({
      packageId: "chromium",
      appName: "Chromium",
      installed: false,
    });

    assert.equal(oldPackageElement.key, "firefox");
    assert.equal(newPackageElement.key, "chromium");
    assert.notEqual(oldPackageElement.key, newPackageElement.key);
  } finally {
    await vite.close();
  }
});
