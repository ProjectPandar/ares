import { test as base } from "@playwright/test";

export const test = base.extend<{ failureDiagnostics: void }>({
  failureDiagnostics: [async ({ page }, use, info) => {
    await page.addInitScript(() => {
      const history: unknown[] = [];
      let previous: Uint8ClampedArray | undefined;
      Object.assign(window, { aresCanvasDiagnostics: history });
      setInterval(() => {
        const canvas = document.querySelector<HTMLCanvasElement>("[data-testid=viewport] canvas");
        if (!canvas) return;
        const probe = document.createElement("canvas");
        probe.width = probe.height = 32;
        const context = probe.getContext("2d")!;
        context.drawImage(canvas, 0, 0, 32, 32);
        const pixels = context.getImageData(0, 0, 32, 32).data;
        let diffPixels = 0;
        const colors = new Set<number>();
        for (let i = 0; i < pixels.length; i += 4) {
          colors.add((pixels[i] << 16) | (pixels[i + 1] << 8) | pixels[i + 2]);
          if (previous && pixels.slice(i, i + 4).some((value, offset) => value !== previous![i + offset])) diffPixels++;
        }
        history.push({ time: performance.now(), width: canvas.width, height: canvas.height,
          dpr: devicePixelRatio, colors: colors.size, diffPixels, sample: Array.from(pixels.slice(0, 16)) });
        if (history.length > 12) history.shift();
        previous = pixels;
      }, 1000);
    });
    await use();
    if (info.status === info.expectedStatus) return;
    try {
      const browser = await page.evaluate(() => {
        const canvas = document.querySelector<HTMLCanvasElement>("[data-testid=viewport] canvas");
        const gl = canvas?.getContext("webgl2");
        const debug = gl?.getExtension("WEBGL_debug_renderer_info");
        return {
          history: (window as unknown as { aresCanvasDiagnostics: unknown[] }).aresCanvasDiagnostics,
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
