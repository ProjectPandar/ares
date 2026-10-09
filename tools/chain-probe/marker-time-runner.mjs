// Shared test-tool Native owner. All output and lifecycle receipts are external.
import { spawn, spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";

export function runNative(binary, command, prefix, deadline) {
  return new Promise((resolve, reject) => {
    const started = performance.now();
    const remaining = Math.ceil(deadline - started);
    if (remaining <= 0) {
      writeFileSync(`${prefix}.status.json`, JSON.stringify({ timedOut: true, command: [binary, ...command] }) + "\n");
      reject(new Error("shared Native deadline exceeded before launch"));
      return;
    }
    const child = spawn(binary, command, { detached: process.platform !== "win32", stdio: ["ignore", "pipe", "pipe"] });
    const stdout = [], stderr = [];
    let timedOut = false, spawnError;
    const stop = () => {
      if (!child.pid) return;
      if (process.platform === "win32") {
        spawnSync("taskkill", ["/PID", String(child.pid), "/T", "/F"]);
      } else {
        try { process.kill(-child.pid, "SIGKILL"); }
        catch (error) { if (error.code !== "ESRCH") throw error; }
      }
    };
    const timer = setTimeout(() => { timedOut = true; stop(); }, remaining);
    child.stdout.on("data", chunk => stdout.push(chunk));
    child.stderr.on("data", chunk => stderr.push(chunk));
    child.on("error", error => { spawnError = error; });
    child.on("close", (exit, signal) => {
      clearTimeout(timer);
      stop();
      writeFileSync(`${prefix}.stdout`, Buffer.concat(stdout));
      writeFileSync(`${prefix}.stderr`, Buffer.concat(stderr));
      writeFileSync(`${prefix}.status.json`, JSON.stringify({
        command: [binary, ...command], pid: child.pid, exit, signal, timedOut,
        error: spawnError?.message, elapsedMs: performance.now() - started,
      }) + "\n");
      if (timedOut) reject(new Error(`shared Native deadline exceeded: ${prefix}.status.json`));
      else if (spawnError) reject(spawnError);
      else if (exit !== 0) reject(new Error(`Native failed status=${exit}: ${prefix}.status.json`));
      else resolve(Buffer.concat(stdout));
    });
  });
}
