import { test as base } from "@playwright/test";

export const test = base.extend<{ failureDiagnostics: void }>({
  failureDiagnostics: [async ({ page }, use, info) => {
    await use();
    if (info.status === info.expectedStatus) return;
    try {
      const browser = await page.evaluate(async () => {
        const canvas = document.querySelector<HTMLCanvasElement>("[data-testid=viewport] canvas");
        const sample = () => {
          if (!canvas) return null;
          const probe = document.createElement("canvas");
          probe.width = probe.height = 32;
          const context = probe.getContext("2d")!;
          context.drawImage(canvas, 0, 0, 32, 32);
          return context.getImageData(0, 0, 32, 32).data;
        };
        const before = sample();
        await new Promise((resolve) => setTimeout(resolve, 100));
        const after = sample();
        let diffPixels = 0;
        const colors = new Set<number>();
        if (before && after) {
          for (let i = 0; i < after.length; i += 4) {
            colors.add((after[i] << 16) | (after[i + 1] << 8) | after[i + 2]);
            if (after.slice(i, i + 4).some((value, offset) => value !== before[i + offset])) diffPixels++;
          }
        }
        const gl = canvas?.getContext("webgl2");
        const debug = gl?.getExtension("WEBGL_debug_renderer_info");
        return {
          width: canvas?.width, height: canvas?.height, colors: colors.size, diffPixels,
          before: before ? Array.from(before.slice(0, 16)) : null,
          after: after ? Array.from(after.slice(0, 16)) : null,
          viewport: [innerWidth, innerHeight], dpr: devicePixelRatio, userAgent: navigator.userAgent,
          locale: navigator.language, timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          renderer: gl?.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER),
          vendor: gl?.getParameter(debug ? debug.UNMASKED_VENDOR_WEBGL : gl.VENDOR),
          contextLost: gl?.isContextLost(), drawing: canvas?.parentElement?.dataset.drawing,
        };
      });
      console.error("ARES_WEB_FAILURE_DIAGNOSTICS", JSON.stringify({
        test: info.title, project: info.project.name, launch: info.project.use.launchOptions,
        display: process.env.DISPLAY ?? null, locale: process.env.LC_ALL,
        tz: process.env.TZ ?? null, browser,
      }));
    } catch (error) {
      console.error("ARES_WEB_FAILURE_DIAGNOSTICS unavailable", String(error));
    }
  }, { auto: true }],
});
