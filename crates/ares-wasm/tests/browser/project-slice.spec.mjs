import { expect, test } from "@playwright/test";

test("sliceProject passes the real 3MF through the generated browser binding", async ({
  page,
}) => {
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.aresReady)).toBe(true);

  const result = await page.evaluate(() => window.sliceFixtureProject());

  expect(result).toEqual({
    resolved: false,
    error: "ProjectSlicingIncomplete",
  });
});
