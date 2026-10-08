import { expect, type Locator, type Page } from "@playwright/test";

// LH system presets (OrcaSlicer/resources/profiles/LH.json), installed through the wizard.
export const PRESETS = {
  vendor: "LH",
  printer: "LH Stinger",
  process: "0.20mm Daily @LH Stinger",
  filament: "LHS PLA",
};

export type Input = "mouse" | "touch";

export const act = (input: Input, control: Locator) => (input === "touch" ? control.tap() : control.click());

export async function choose(page: Page, input: Input, combo: string, group: string, name: string) {
  await act(input, page.getByRole("button", { name: combo, exact: true }));
  await act(input, page.getByRole("listbox", { name: combo, exact: true }).getByRole("group", { name: group })
    .getByRole("option", { name, exact: true }));
  await expect(page.getByRole("button", { name: combo, exact: true })).toHaveText(name);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}
