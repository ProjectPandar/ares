import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  plugins: [react()],
  worker: { format: "es" },
  // The lazily loaded Three.js viewport chunk is ~590 kB; the slicer itself ships as .wasm.
  build: { target: "es2022", chunkSizeWarningLimit: 650 },
  server: { port: 5173 },
  preview: { port: 4174, strictPort: true },
});
