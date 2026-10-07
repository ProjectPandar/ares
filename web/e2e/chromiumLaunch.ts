// Hosts without Playwright's bundled browser libraries (e.g. NixOS) point
// ARES_E2E_CHROME at a system Chrome/Chromium.
export const chromiumLaunch = {
  executablePath: process.env.ARES_E2E_CHROME || undefined,
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
};
