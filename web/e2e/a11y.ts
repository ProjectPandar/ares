import type { Locator, Page } from "@playwright/test";

/** Visible text fields and selects under 16px, which iOS Safari zooms into on focus. */
export const smallFonts = (page: Page) => page.evaluate(() =>
  [...document.querySelectorAll("input, select, textarea")]
    .filter((element) => !["checkbox", "radio", "range", "file", "hidden"].includes((element as HTMLInputElement).type)
      && element.getClientRects().length > 0 && parseFloat(getComputedStyle(element).fontSize) < 16)
    .map((element) => `${element.tagName.toLowerCase()}.${element.className} "${element.getAttribute("aria-label")}" ${getComputedStyle(element).fontSize}`));

/**
 * Elements (and their pseudo-elements) still animating or transitioning for
 * longer than 1ms. A detached `.spinner` probe covers the loading spinner when
 * nothing is loading.
 */
export const moving = (page: Page) => page.evaluate(() => {
  const seconds = (list: string) => Math.max(...list.split(",").map((d) => parseFloat(d) / (d.trim().endsWith("ms") ? 1000 : 1)));
  const busy = (style: CSSStyleDeclaration) =>
    (style.animationName !== "none" && seconds(style.animationDuration) > 0.001) || seconds(style.transitionDuration) > 0.001;
  const probe = document.body.appendChild(Object.assign(document.createElement("span"), { className: "spinner" }));
  const found = [...document.querySelectorAll("body *")].flatMap((element) => [null, "::before", "::after"]
    .filter((pseudo) => busy(getComputedStyle(element, pseudo)))
    .map((pseudo) => `${element.tagName.toLowerCase()}.${element.getAttribute("class") ?? ""}${pseudo ?? ""}`));
  probe.remove();
  return [...new Set(found)];
});

/**
 * Tabs through the page from `start` until focus repeats. Returns the names
 * of the controls reached and of those whose focused look equals their
 * unfocused look (no visible focus indicator). It starts on a control because
 * Tab on the body switches Prepare/Preview (GLCanvas3D::on_key, Shell.tsx).
 */
export async function tabSweep(page: Page, start: Locator) {
  await start.focus();
  const reached: string[] = [];
  const invisible: string[] = [];
  for (let step = 0; step < 400; step += 1) {
    await page.keyboard.press("Tab");
    const stop = await page.evaluate(() => {
      const element = document.activeElement as HTMLElement | null;
      const seen = ((window as unknown as { tabSeen?: Set<Element> }).tabSeen ??= new Set());
      if (!element || element === document.body || seen.has(element)) return null;
      seen.add(element);
      const look = () => {
        const s = getComputedStyle(element);
        return [s.outlineStyle, s.outlineWidth, s.outlineColor, s.boxShadow, s.borderColor, s.backgroundColor, s.color].join(" ");
      };
      const focused = look();
      element.blur();
      const unfocused = look();
      element.focus();
      const name = element.getAttribute("aria-label") || element.title || element.textContent?.trim();
      return { name: `${element.tagName.toLowerCase()} "${name}"`, visible: focused !== unfocused };
    });
    if (!stop) break;
    reached.push(stop.name);
    if (!stop.visible) invisible.push(stop.name);
  }
  await page.evaluate(() => {
    delete (window as unknown as { tabSeen?: Set<Element> }).tabSeen;
    (document.activeElement as HTMLElement | null)?.blur();
  });
  return { reached, invisible };
}
