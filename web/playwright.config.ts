import { defineConfig, devices } from "@playwright/test";
import { chromiumLaunch as chromium } from "./e2e/chromiumLaunch";

// Touch-emulated phones and tablets run only the mobile flow.
const webkit = { executablePath: process.env.ARES_E2E_WEBKIT || undefined };
const mobileSpecs = /mobile\.spec\.ts$/;

export default defineConfig({
  testDir: "e2e",
  outputDir: "../target/web-e2e/test-results",
  reporter: "line",
  timeout: 300_000,
  use: {
    baseURL: "http://127.0.0.1:4174",
    headless: true,
  },
  projects: [
    {
      name: "desktop",
      testIgnore: /mobile\.spec\.ts$/,
      use: { browserName: "chromium", viewport: { width: 1600, height: 900 }, launchOptions: chromium },
    },
    {
      // Bundled WebKit; on NixOS run through e2e/webkit-env.sh.
      name: "iphone",
      testMatch: mobileSpecs,
      use: { ...devices["iPhone 15"], launchOptions: webkit, actionTimeout: 30_000 },
    },
    {
      name: "pixel",
      testMatch: mobileSpecs,
      use: { ...devices["Pixel 7"], launchOptions: chromium, actionTimeout: 30_000 },
    },
    {
      // 810px wide: the tablet layout (641-1024px).
      name: "ipad",
      testMatch: mobileSpecs,
      use: { ...devices["iPad (gen 7)"], launchOptions: webkit, actionTimeout: 30_000 },
    },
  ],
  webServer: {
    command: "npm run preview -w ares-app -- --host 127.0.0.1",
    url: "http://127.0.0.1:4174/",
    reuseExistingServer: false,
  },
});
