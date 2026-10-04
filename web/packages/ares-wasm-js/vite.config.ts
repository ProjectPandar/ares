import { copyFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const wasmFile = fileURLToPath(new URL("src/wasm/ares_wasm_bg.wasm", import.meta.url));

export default defineConfig({
  build: {
    lib: {
      entry: { index: "src/index.ts", worker: "src/worker.ts" },
      formats: ["es"],
    },
    target: "es2022",
    assetsInlineLimit: 0,
    emptyOutDir: true,
  },
  plugins: [
    {
      name: "ares-copy-wasm",
      apply: "build",
      writeBundle(options) {
        copyFileSync(wasmFile, `${options.dir}/ares_wasm_bg.wasm`);
      },
    },
  ],
});
