import { defineConfig } from "@playwright/test";
import { fileURLToPath } from "node:url";

export default defineConfig({
  testDir: fileURLToPath(new URL(".", import.meta.url)),
  outputDir: fileURLToPath(
    new URL("../../../../target/playwright-results", import.meta.url),
  ),
  reporter: "line",
  use: {
    baseURL: "http://127.0.0.1:4173",
    browserName: "chromium",
    headless: true,
  },
  webServer: {
    command: "node server.mjs",
    url: "http://127.0.0.1:4173/index.html",
    reuseExistingServer: false,
  },
});
