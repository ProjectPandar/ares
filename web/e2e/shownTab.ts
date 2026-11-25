import type { Page } from "@playwright/test";

/** The shown tab pane, Prepare or Preview; the hidden pane keeps its elements and test ids. */
export const shownTab = (page: Page) => page.locator(".tab-pane:not([hidden])");
