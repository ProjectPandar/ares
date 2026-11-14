import { readFileSync } from "node:fs";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The release version: package.json's, which every batch release bumps.
const { version } = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")) as { version: string };

export default defineConfig({
  base: "./",
  plugins: [react()],
  define: { ARES_VERSION: JSON.stringify(version) },
  worker: { format: "es" },
  // The lazily loaded Three.js viewport chunk is ~590 kB; the slicer itself ships as .wasm.
  build: { target: "es2022", chunkSizeWarningLimit: 650 },
  server: { port: 5173 },
  preview: { port: 4174, strictPort: true },
});
