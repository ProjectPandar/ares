// Failure inventory: timeout must kill the owned process tree and fail with a
// receipt; nonzero/spawn errors must retain channels and fail, never become skips.
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { runNative } from "./marker-time-runner.mjs";

test("timeout kills owned Native descendants and retains failure receipt", async () => {
  const prefix = join(mkdtempSync(join(tmpdir(), "marker-native-timeout-")), "native");
  await assert.rejects(runNative(process.execPath, ["-e", `
    const {spawn} = require('node:child_process');
    const child = spawn(process.execPath, ['-e', 'setInterval(()=>{},1000)'], {stdio:'ignore'});
    console.log(child.pid);
    setInterval(()=>{},1000);
  `], prefix, performance.now() + 1000), /deadline/);
  const receipt = JSON.parse(readFileSync(`${prefix}.status.json`, "utf8"));
  assert.equal(receipt.timedOut, true);
  assert.equal(receipt.exit, null);
  const pid = Number(readFileSync(`${prefix}.stdout`, "utf8").trim());
  assert(pid > 0, "owned child was not exercised");
  if (process.platform === "linux") {
    // A killed orphan may wait for init to reap; it must not remain running.
    try { assert.equal(readFileSync(`/proc/${pid}/stat`, "utf8").split(' ')[2], 'Z'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
});

test("nonzero is failure and captures stdout, stderr and status", async () => {
  const prefix = join(mkdtempSync(join(tmpdir(), "marker-native-nonzero-")), "native");
  await assert.rejects(runNative(process.execPath, ["-e", "console.log('out');console.error('err');process.exit(17)"], prefix, performance.now() + 3000), /status=17/);
  assert.equal(readFileSync(`${prefix}.stdout`, "utf8"), "out\n");
  assert.equal(readFileSync(`${prefix}.stderr`, "utf8"), "err\n");
  assert.equal(JSON.parse(readFileSync(`${prefix}.status.json`, "utf8")).exit, 17);
});
