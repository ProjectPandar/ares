import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";

test("a system printer model shows the filtering state from its selection event, as Plater::priv::on_select_preset selects it in the event", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  const printer = page.getByRole("button", { name: "Printer preset", exact: true });
  const filtering = page.getByTestId("preset-filtering");
  await printer.click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await expect(filtering).toHaveCount(0, { timeout: 60_000 });

  await printer.click();
  const model = page.getByRole("listbox", { name: "Printer preset", exact: true }).getByRole("group", { name: "System presets" })
    .getByRole("option", { name: "Bambu Lab A1 mini", exact: true });
  // Engine calls answer in later tasks, so one microtask after the click shows only what the selection event set.
  const filteringAtSelection = await model.evaluate(async (option: HTMLElement) => {
    option.click();
    await new Promise<void>(queueMicrotask);
    return document.querySelector("[data-testid=preset-filtering]") !== null;
  });
  expect(filteringAtSelection, "filtering state right after the selection event").toBe(true);
  await expect(printer).toHaveAttribute("title", "Bambu Lab A1 mini 0.4 nozzle", { timeout: 60_000 });
  await expect(filtering).toHaveCount(0, { timeout: 60_000 });
});
