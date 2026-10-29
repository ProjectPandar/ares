import { defineConfig } from "@playwright/test";
import { chromiumLaunch } from "./e2e/chromiumLaunch";

export default defineConfig({
  testDir: "pages",
  outputDir: "../target/pages-e2e",
  reporter: [["line"], ["json", { outputFile: "../target/pages-e2e/results.json" }]],
  timeout: 120_000,
  use: { baseURL: "http://127.0.0.1:4175", launchOptions: chromiumLaunch, trace: "retain-on-failure" },
  webServer: {
    command: "wrangler pages dev app/dist --ip 127.0.0.1 --port 4175 --compatibility-date 2026-10-06",
    url: "http://127.0.0.1:4175",
    reuseExistingServer: false,
  },
});
