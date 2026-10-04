import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "e2e",
  outputDir: "../target/web-e2e/test-results",
  reporter: "line",
  timeout: 300_000,
  use: {
    baseURL: "http://127.0.0.1:4174",
    browserName: "chromium",
    headless: true,
    viewport: { width: 1600, height: 900 },
    launchOptions: {
      // Hosts without Playwright's bundled browser libraries (e.g. NixOS) point
      // this at a system Chrome/Chromium.
      executablePath: process.env.ARES_E2E_CHROME || undefined,
      args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
    },
  },
  webServer: {
    command: "npm run preview -w ares-app -- --host 127.0.0.1",
    url: "http://127.0.0.1:4174/",
    reuseExistingServer: false,
  },
});
