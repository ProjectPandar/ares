import { expect, type Locator, type Page } from "@playwright/test";

type Point = { x: number; y: number };
/** One gesture frame: the position of every finger. */
type Frame = Point[];

/**
 * Plays a multi-finger touch gesture. Chromium gets real touches through
 * CDP `Input.dispatchTouchEvent`, so `touch-action` and default panning apply
 * as on a phone. Playwright has no WebKit touch-move input, so WebKit gets
 * the same frames as `touch` pointer events dispatched at the element under
 * the first finger; `touch-action` is not exercised there. Those pointer ids
 * are not active browser pointers, so `setPointerCapture` would throw
 * NotFoundError; capture calls for them are no-ops while the frames play.
 */
export async function gesture(page: Page, frames: Frame[]) {
  const browser = page.context().browser()!.browserType().name();
  if (browser === "chromium") {
    const cdp = await page.context().newCDPSession(page);
    const points = (frame: Frame) => frame.map(({ x, y }, id) => ({ x, y, id }));
    await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: points(frames[0]) });
    for (const frame of frames.slice(1)) {
      await cdp.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: points(frame) });
    }
    await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
    await cdp.detach();
    return "cdp-touch";
  }
  await page.evaluate((played) => {
    const target = document.elementFromPoint(played[0][0].x, played[0][0].y)!;
    const { setPointerCapture, releasePointerCapture } = Element.prototype;
    const synthetic = (id: number) => id >= 100 && id < 100 + played[0].length;
    Element.prototype.setPointerCapture = function (id) { if (!synthetic(id)) setPointerCapture.call(this, id); };
    Element.prototype.releasePointerCapture = function (id) { if (!synthetic(id)) releasePointerCapture.call(this, id); };
    const fire = (type: string, frame: Point[]) => frame.forEach(({ x, y }, index) => target.dispatchEvent(new PointerEvent(type, {
      pointerId: 100 + index, pointerType: "touch", isPrimary: index === 0, clientX: x, clientY: y,
      bubbles: true, cancelable: true, composed: true, buttons: type === "pointerup" ? 0 : 1, button: 0,
    })));
    fire("pointerdown", played[0]);
    for (const frame of played.slice(1)) fire("pointermove", frame);
    fire("pointerup", played.at(-1)!);
    Object.assign(Element.prototype, { setPointerCapture, releasePointerCapture });
  }, frames);
  return "synthetic-pointer";
}

const steps = (count: number) => Array.from({ length: count + 1 }, (_, index) => index / count);

/** One finger from `from` to `to`. */
export const swipe = (page: Page, from: Point, to: Point) =>
  gesture(page, steps(10).map((t) => [{ x: from.x + (to.x - from.x) * t, y: from.y + (to.y - from.y) * t }]));

/** Two fingers moving apart symmetrically around `center` from `start` to `end` px apart. */
export const pinch = (page: Page, center: Point, start: number, end: number) =>
  gesture(page, steps(10).map((t) => {
    const half = (start + (end - start) * t) / 2;
    return [{ x: center.x - half, y: center.y }, { x: center.x + half, y: center.y }];
  }));

/** WCAG 2.5.5 target size, the 44 CSS px the mobile task requires. */
export const MIN_TARGET = 44;

/**
 * Every visible control on the page that is smaller than 44x44, as `name WxH`. A checkbox or
 * radio counts its label, which activates it too.
 */
export const undersized = (page: Page) => page.evaluate((min) => {
  const controls = document.querySelectorAll<HTMLElement>(
    "button, a[href], input:not([type=hidden]), select, textarea, [role=slider], [role=option], [role=tab], [role=checkbox]");
  return [...controls].flatMap((control) => {
    const target = control.matches("input[type=checkbox], input[type=radio]") ? control.closest("label") ?? control : control;
    const { width, height } = target.getBoundingClientRect();
    if (!width || !height || getComputedStyle(target).visibility !== "visible") return [];
    if (width >= min && height >= min) return [];
    const name = control.getAttribute("aria-label") ?? control.textContent?.trim().slice(0, 30) ?? "";
    return [`${control.tagName.toLowerCase()}.${control.className} "${name}" ${Math.round(width)}x${Math.round(height)}`];
  });
}, MIN_TARGET);

/**
 * The page does not scroll sideways and every control is fully on screen, uncovered and at least
 * 44x44. Soft assertions, so one run reports every misfit.
 */
export async function expectFits(page: Page, controls: Locator[]) {
  const { scrollWidth, innerWidth } = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    innerWidth: window.innerWidth,
  }));
  expect.soft(scrollWidth, "no horizontal scrolling").toBeLessThanOrEqual(innerWidth);
  const viewport = page.viewportSize()!;
  for (const control of controls) {
    const name = `${control}`;
    await expect.soft(control, `${name} is visible`).toBeVisible();
    if (!(await control.isVisible())) continue;
    const box = (await control.boundingBox())!;
    expect.soft(box.x, `${name} starts on screen`).toBeGreaterThanOrEqual(0);
    expect.soft(box.y, `${name} starts on screen`).toBeGreaterThanOrEqual(0);
    expect.soft(box.x + box.width, `${name} ends on screen`).toBeLessThanOrEqual(viewport.width);
    expect.soft(box.y + box.height, `${name} ends on screen`).toBeLessThanOrEqual(viewport.height);
    expect.soft(box.width, `${name} target width`).toBeGreaterThanOrEqual(MIN_TARGET);
    expect.soft(box.height, `${name} target height`).toBeGreaterThanOrEqual(MIN_TARGET);
    // Nothing covers the control's centre.
    const covered = await control.evaluate((element, [x, y]) => {
      const top = document.elementFromPoint(x, y);
      return !top || !(element === top || element.contains(top));
    }, [box.x + box.width / 2, box.y + box.height / 2]);
    expect.soft(covered, `${name} is not covered`).toBe(false);
  }
}
