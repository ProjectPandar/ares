import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  plugins: [react()],
  worker: { format: "es" },
  // Three.js dominates the single app chunk; the slicer itself ships as .wasm.
  build: { target: "es2022", chunkSizeWarningLimit: 1200 },
  server: { port: 5173 },
  preview: { port: 4174, strictPort: true },
});
