import { PresetCatalog } from "./wasm/ares_wasm.js";
import type { WorkerRequest } from "./protocol.js";

/** Cached handle for the most recent raw `catalogJson` string caller (reused across repeated calls). */
let cachedJsonCatalog: { json: string; handle: PresetCatalog } | undefined;

/** Returns the worker's WASM `PresetCatalog` handle for a `catalogId` or a cached `catalogJson` string. */
export function catalogHandleFor(source: string | number, catalogs: Map<number, PresetCatalog>): PresetCatalog {
  if (typeof source === "number") return catalogs.get(source)!;
  if (cachedJsonCatalog?.json === source) return cachedJsonCatalog.handle;
  const handle = PresetCatalog.fromJson(source);
  cachedJsonCatalog?.handle.free();
  cachedJsonCatalog = { json: source, handle };
  return handle;
}

export type CatalogRequest = Extract<
  WorkerRequest,
  {
    kind:
      | "catalogCreate"
      | "catalogAddVendor"
      | "catalogAddImport"
      | "catalogFilter"
      | "catalogPrinters"
      | "catalogPrinterModels"
      | "catalogSimilarPrinter"
      | "catalogPresetValues"
      | "catalogPrinterModelInfo"
      | "catalogInstalledFilaments"
      | "catalogSelectCompatibleFilaments"
      | "catalogNormalizeFilamentSlots"
      | "catalogSetInstalledPrinters"
      | "catalogSystemPrinter"
      | "catalogCompatiblePrint"
      | "catalogGuideFilaments"
      | "catalogFree";
  }
>;

export function handleCatalog(
  request: CatalogRequest,
  catalogs: Map<number, PresetCatalog>,
): [unknown, Transferable[]] {
  switch (request.kind) {
    case "catalogCreate":
      catalogs.set(request.catalogId, new PresetCatalog());
      return [undefined, []];
    case "catalogAddVendor":
      catalogs.get(request.catalogId)!.addVendor(request.input);
      return [undefined, []];
    case "catalogAddImport":
      catalogs.get(request.catalogId)!.addImport(request.input, request.filename);
      return [undefined, []];
    case "catalogFilter":
      return [JSON.parse(catalogs.get(request.catalogId)!.filterCompatible(request.printer, request.process)), []];
    case "catalogPrinters":
      return [JSON.parse(catalogs.get(request.catalogId)!.printers()), []];
    case "catalogPrinterModels":
      return [JSON.parse(catalogs.get(request.catalogId)!.printerModels()), []];
    case "catalogSimilarPrinter": {
      const { catalogId, model, variant, current } = request;
      return [
        JSON.parse(
          catalogs.get(catalogId)!.similarPrinterPreset(model, variant, current.name, current.printer_variant),
        ),
        [],
      ];
    }
    case "catalogPresetValues": {
      const { catalogId, presetKind, names, keys } = request;
      return [
        JSON.parse(catalogs.get(catalogId)!.presetValues(presetKind, JSON.stringify(names), JSON.stringify(keys))),
        [],
      ];
    }
    case "catalogPrinterModelInfo":
      return [JSON.parse(catalogs.get(request.catalogId)!.printerModelInfo(request.model)), []];
    case "catalogInstalledFilaments":
      return [JSON.parse(catalogs.get(request.catalogId)!.installedFilaments(JSON.stringify(request.installed))), []];
    case "catalogSelectCompatibleFilaments":
      return [
        JSON.parse(
          catalogs
            .get(request.catalogId)!
            .selectCompatibleFilaments(
              request.printer,
              request.process,
              JSON.stringify(request.filaments),
              JSON.stringify(request.installed),
            ),
        ),
        [],
      ];
    case "catalogNormalizeFilamentSlots":
      return [
        JSON.parse(
          catalogs.get(request.catalogId)!.normalizeFilamentSlots(request.printer, JSON.stringify(request.state)),
        ),
        [],
      ];
    case "catalogSetInstalledPrinters":
      catalogs.get(request.catalogId)!.setInstalledPrinters(JSON.stringify(request.installed));
      return [undefined, []];
    case "catalogSystemPrinter":
      return [JSON.parse(catalogs.get(request.catalogId)!.systemPrinter(request.model, request.variant)), []];
    case "catalogCompatiblePrint":
      return [JSON.parse(catalogs.get(request.catalogId)!.compatiblePrint(request.printer, request.current)), []];
    case "catalogGuideFilaments":
      return [JSON.parse(catalogs.get(request.catalogId)!.guideFilaments()), []];
    case "catalogFree":
      catalogs.get(request.catalogId)!.free();
      catalogs.delete(request.catalogId);
      return [undefined, []];
  }
}
