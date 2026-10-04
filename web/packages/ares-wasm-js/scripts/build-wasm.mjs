// Builds crates/ares-wasm for wasm32 and runs wasm-bindgen into ./src/wasm.
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repositoryRoot = fileURLToPath(new URL("../../../../", import.meta.url));
const outDir = fileURLToPath(new URL("../src/wasm/", import.meta.url));
const run = (command, args) =>
  execFileSync(command, args, { cwd: repositoryRoot, stdio: "inherit" });

run("cargo", [
  "build",
  "-p",
  "ares-wasm",
  "--target",
  "wasm32-unknown-unknown",
  "--release",
]);
run("wasm-bindgen", [
  "--target",
  "web",
  "--omit-default-module-path",
  "--out-dir",
  outDir,
  "target/wasm32-unknown-unknown/release/ares_wasm.wasm",
]);
