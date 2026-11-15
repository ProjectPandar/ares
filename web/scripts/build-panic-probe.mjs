// Builds the E2E probe engine into target/web-e2e/panic-probe: crates/ares-wasm with the
// `panic-probe` feature, and the package worker bundled against that build's bindings, whose
// closure exports are numbered for its own module.
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { build } from "vite";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const outDir = `${repositoryRoot}target/web-e2e/panic-probe`;
const run = (command, args) => execFileSync(command, args, { cwd: repositoryRoot, stdio: "inherit" });

run("cargo", ["build", "-p", "ares-wasm", "--target", "wasm32-unknown-unknown", "--release", "--features", "panic-probe"]);
run("wasm-bindgen", ["--target", "web", "--omit-default-module-path", "--out-dir", outDir, "target/wasm32-unknown-unknown/release/ares_wasm.wasm"]);
await build({
  configFile: false,
  publicDir: false,
  logLevel: "warn",
  resolve: { alias: [{ find: /^\.\/wasm\/ares_wasm\.js$/, replacement: `${outDir}/ares_wasm.js` }] },
  build: {
    outDir,
    emptyOutDir: false,
    lib: { entry: `${repositoryRoot}web/packages/ares-wasm-js/src/worker.ts`, formats: ["es"], fileName: () => "worker.js" },
  },
});
