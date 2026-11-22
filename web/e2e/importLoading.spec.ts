import { readFileSync } from "node:fs";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";

const first = readFileSync(new URL("../../tests/parity/cube10.stl", import.meta.url));
const second = readFileSync(new URL("../../tests/parity/arachne/short-branch.stl", import.meta.url));

declare global {
  interface Window {
    importRead: { finish: ((ok: boolean) => void) | null };
  }
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const read = File.prototype.arrayBuffer;
    const relay: Window["importRead"] = { finish: null };
    window.importRead = relay;
    File.prototype.arrayBuffer = function () {
      if (this.name !== "second.stl") return read.call(this);
      return new Promise<void>((resolve, reject) => {
        relay.finish = (ok: boolean) => {
          if (ok) resolve();
          else reject(new Error("Could not read the file"));
        };
      }).then(() => read.call(this));
    };
  });
});

for (const ok of [true, false]) {
  test(ok ? "an import prevents selection until the replacement scene is ready" :
    "a rejected file read reports the error and restores selection in the old scene", async ({ page }, info) => {
    const status = page.getByTestId("status");
    const picker = page.getByTestId("import-input");
    const row = (name: string) => page.getByTestId("object-list").locator('[role="treeitem"]').filter({ hasText: name });
    const previous = row("first.stl");
    const current = row(ok ? "second.stl" : "first.stl");
    const errors: string[] = [];
    page.on("pageerror", error => errors.push(error.message));

    await page.goto("/");
    await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
    await picker.setInputFiles({ name: "first.stl", mimeType: "model/stl", buffer: first });
    await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });
    const panel = page.getByRole("button", { name: "Settings panel", exact: true });
    if (await panel.isVisible()) await panel.click();
    await expect(previous).toHaveAttribute("aria-selected", "false");
    await previous.scrollIntoViewIfNeeded();
    const box = (await previous.boundingBox())!;

    await picker.setInputFiles({ name: "second.stl", mimeType: "model/stl", buffer: second });
    await page.waitForFunction(() => window.importRead.finish !== null);
    await expect.soft(status).toHaveText("Loading model…", { timeout: 2_000 });
    const [x, y] = [box.x + box.width / 2, box.y + box.height / 2];
    if (info.project.use.hasTouch) await page.touchscreen.tap(x, y);
    else await page.mouse.click(x, y);
    expect.soft(await previous.getAttribute("aria-selected")).toBe("false");

    await page.evaluate((success) => window.importRead.finish!(success), ok);
    await expect(status).toHaveText(ok ? "Ready to slice" : "Import failed: Could not read the file", { timeout: 10_000 });
    if (ok) await expect(previous).toHaveCount(0);
    await expect(current).toHaveAttribute("aria-selected", "false");
    await current.click();
    await expect(current).toHaveAttribute("aria-selected", "true");
    expect(errors).toEqual([]);
  });
}
