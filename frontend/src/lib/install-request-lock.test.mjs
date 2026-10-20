import assert from "node:assert/strict";
import test from "node:test";
import { InstallRequestLock } from "./install-request-lock.ts";

test("blocks a second install request for the same package", () => {
  const lock = new InstallRequestLock();

  assert.equal(lock.tryAcquire("firefox"), true);
  assert.equal(lock.isLocked("firefox"), true);
  assert.equal(lock.tryAcquire("firefox"), false);
  assert.equal(lock.tryAcquire("chromium"), true);

  lock.release("firefox");
  assert.equal(lock.isLocked("firefox"), false);
  assert.equal(lock.tryAcquire("firefox"), true);
});

test("releasing an unknown package is harmless", () => {
  const lock = new InstallRequestLock();

  assert.equal(lock.isLocked("firefox"), false);
  lock.release("firefox");
  assert.equal(lock.tryAcquire("firefox"), true);
});
