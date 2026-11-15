import { test as base, type BrowserContext, type Page } from "@playwright/test";

/**
 * The app config after a first run's setup wizard: the stored result of checking only Custom /
 * Generic Klipper Printer and pressing Finish, which printerWizard.spec.ts checks against a real
 * Finish. It is not upstream's cancel path (WebGuideDialog.cpp:1257-1266): that path calls
 * set_variant("Custom", ORCA_DEFAULT_PRINTER_MODEL, "0.4"), but ORCA_DEFAULT_PRINTER_MODEL is the
 * preset name "MyKlipper 0.4 nozzle" (PresetBundle.cpp:187-189), not its printer_model "Generic
 * Klipper Printer" (Custom/machine/MyKlipper 0.4 nozzle.json:27), so it installs no visible printer.
 */
export const FIRST_RUN_CONFIG = {
  version: 1,
  theme: "dark",
  mode: "simple",
  gcodeWindow: true,
  installedPrinters: { Custom: { "Generic Klipper Printer": ["0.2", "0.4", "0.6", "0.8"] } },
  printer: "MyKlipper 0.4 nozzle",
  // load_selections takes the printer's default_filament_profile (PresetBundle.cpp:4070-4076, 4163;
  // Custom/machine/fdm_klipper_common.json:230-232); apply_vendor_config keeps it as it is checked (:2079-2095).
  // The print is update_compatible's PreferedPrintProfileMatch pick (PresetBundle.cpp:8635-8639): the printer's
  // default_print_profile (Custom/machine/fdm_klipper_common.json:233).
  // The bed type is load_selections' pick for a printer without a saved one (PresetBundle.cpp:4046-4054):
  // get_default_bed_type finds no default_bed_type in the MyKlipper presets (default "", PrintConfig.cpp:1475) and a
  // printer type other than the BBL models, so btPEI (Preset.cpp:1140-1171), "High Temp Plate" (PrintConfig.cpp:611);
  // export_selections stores it per printer (PresetBundle.cpp:4255).
  printers: {
    "MyKlipper 0.4 nozzle": { filaments: ["Generic PLA @System"], process: "0.20mm Standard @MyKlipper", bedType: "High Temp Plate" },
  },
  // The model's default_materials (Custom/machine/Generic Klipper Printer.json), checked by save_userguide_models.
  installedFilaments: [
    "Generic ABS @System", "Generic ASA @System", "Generic PA @System", "Generic PA-CF @System", "Generic PC @System",
    "Generic PETG @System", "Generic PLA @System", "Generic PLA-CF @System", "Generic PVA @System", "Generic TPU @System",
  ],
  nozzleVolumeTypes: {},
  preferredFilamentMapMode: "Auto For Flush",
};

/** Pages of `target` without an app config start with `FIRST_RUN_CONFIG`. */
export const seedFirstRun = (target: Page | BrowserContext) =>
  target.addInitScript((record) => {
    if (localStorage.getItem("ares.config") === null) localStorage.setItem("ares.config", record);
  }, JSON.stringify(FIRST_RUN_CONFIG));

export const test = base.extend<{ failureDiagnostics: void; firstRunConfig: boolean; firstRunSeed: void }>({
  /** Whether a page without an app config starts with `FIRST_RUN_CONFIG` instead of the setup wizard. */
  firstRunConfig: [true, { option: true }],
  firstRunSeed: [async ({ page, firstRunConfig }, use) => {
    if (firstRunConfig) {
      await seedFirstRun(page);
    }
    await use();
  }, { auto: true }],
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
