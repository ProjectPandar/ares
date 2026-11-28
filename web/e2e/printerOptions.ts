import { expect, type Locator } from "@playwright/test";

export async function openPrinterOptions(trigger: Locator) {
  await trigger.click();
  const menu = trigger.page().getByRole("menu", { exact: true });
  await expect(menu).toBeVisible();
  return menu.getByRole("menuitemradio");
}

export async function choosePrinterOption(trigger: Locator, label: string) {
  const items = await openPrinterOptions(trigger);
  await items.and(trigger.page().getByRole("menuitemradio", { name: label, exact: true })).click();
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
}
