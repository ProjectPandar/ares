/**
 * Browser-only AppConfig: the selections and preferences OrcaSlicer keeps in its
 * app config (PresetBundle::export_selections, PresetBundle.cpp:2917-2980), in one
 * versioned localStorage record, and the imported preset files in IndexedDB. A
 * record or import entry that does not match its schema is deleted.
 */
import type { Theme, ViewMode } from "./store";

const CONFIG_KEY = "ares.config";
const VERSION = 1;

/** Per-printer settings (AppConfig::set_printer_setting, AppConfig.hpp:239-249). */
export interface PrinterSettings {
  /** `print` */
  process?: string;
  /** `filament`, `filament_01`, ... */
  filaments: string[];
  /** `curr_bed_type`, a `curr_bed_type` enum value. */
  bedType?: string;
}

export interface StoredConfig {
  theme: Theme;
  /** `user_mode` (GUI_App.cpp:7775-7802). */
  mode: ViewMode;
  /** `show_gcode_window` (AppConfig.cpp:301-302). */
  gcodeWindow: boolean;
  /** Installed system vendors (ConfigWizard). */
  vendors: string[];
  /** Section `presets`: the selected printer, if a preset printer is selected. */
  printer: string | null;
  /** Printer settings by printer preset name. */
  printers: Record<string, PrinterSettings>;
  /** Section `filaments`: the installed system filaments (PresetBundle::load_installed_filaments). */
  installedFilaments: string[];
  /** Section `nozzle_volume_types` by printer (AppConfig.cpp:1555-1580). */
  nozzleVolumeTypes: Record<string, string[]>;
  /** `prefered_filament_map_mode` (FilamentGroupPopup.cpp:25-48). */
  preferredFilamentMapMode: "Auto For Flush" | "Auto For Match" | "Manual";
  /** Section `arrange` (GLCanvas3D.cpp:1526-1561), app-config strings by key; absent in older records. */
  arrange?: Record<string, string>;
}

const DEFAULTS: StoredConfig = {
  theme: "dark",
  // OrcaSlicer starts in simple mode (GUI_App::get_saved_mode).
  mode: "simple",
  gcodeWindow: true,
  vendors: [],
  printer: null,
  printers: {},
  installedFilaments: [],
  nozzleVolumeTypes: {},
  // AppConfig.cpp:456-457.
  preferredFilamentMapMode: "Auto For Flush",
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);
const isStrings = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every((item) => typeof item === "string");
const isOptionalString = (value: unknown) => value === undefined || typeof value === "string";

function isPrinterSettings(value: unknown): value is PrinterSettings {
  return isRecord(value) && isOptionalString(value.process) && isStrings(value.filaments) && isOptionalString(value.bedType);
}

function isStoredConfig(value: Record<string, unknown>): boolean {
  return (value.theme === "dark" || value.theme === "light")
    && (value.mode === "simple" || value.mode === "advanced" || value.mode === "expert")
    && typeof value.gcodeWindow === "boolean"
    && isStrings(value.vendors)
    && (value.printer === null || typeof value.printer === "string")
    && isRecord(value.printers) && Object.values(value.printers).every(isPrinterSettings)
    && isStrings(value.installedFilaments)
    && isRecord(value.nozzleVolumeTypes) && Object.values(value.nozzleVolumeTypes).every(isStrings)
    && ["Auto For Flush", "Auto For Match", "Manual"].includes(value.preferredFilamentMapMode as string)
    && (value.arrange === undefined || (isRecord(value.arrange) && Object.values(value.arrange).every((item) => typeof item === "string")));
}

/** The stored record, or the defaults after deleting a missing, old or malformed one. */
function loadConfig(): StoredConfig {
  // Unversioned keys of earlier builds are dropped, not migrated.
  for (const key of ["ares.theme", "ares.mode", "ares.gcodeWindow", "ares.vendors"]) localStorage.removeItem(key);
  const text = localStorage.getItem(CONFIG_KEY);
  if (text !== null) {
    try {
      const { version, ...config } = JSON.parse(text) as Record<string, unknown>;
      if (version === VERSION && isStoredConfig(config)) return config as unknown as StoredConfig;
    } catch {
      // Not JSON: dropped below.
    }
    localStorage.removeItem(CONFIG_KEY);
  }
  return DEFAULTS;
}

export const storedConfig: StoredConfig = loadConfig();

export function saveConfig(patch: Partial<StoredConfig>) {
  Object.assign(storedConfig, patch);
  localStorage.setItem(CONFIG_KEY, JSON.stringify({ version: VERSION, ...storedConfig }));
}

/** An imported preset file (MainFrame::load_config_file, MainFrame.cpp:3723-3745). */
export interface StoredImport {
  filename: string;
  bytes: Uint8Array;
}

const DB = "ares";
const IMPORTS = "presetImports";

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB, VERSION);
    request.onupgradeneeded = () => request.result.createObjectStore(IMPORTS, { autoIncrement: true });
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error!);
  });
}

function done(transaction: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error!);
  });
}

const isStoredImport = (value: unknown): value is { version: number } & StoredImport =>
  isRecord(value) && value.version === VERSION && typeof value.filename === "string" && value.bytes instanceof Uint8Array;

/** Imports in import order with their keys; malformed entries are deleted. */
export async function loadImports(): Promise<{ key: IDBValidKey; file: StoredImport }[]> {
  const db = await openDb();
  const transaction = db.transaction(IMPORTS, "readwrite");
  const store = transaction.objectStore(IMPORTS);
  const entries: { key: IDBValidKey; file: StoredImport }[] = [];
  store.openCursor().onsuccess = (event) => {
    const cursor = (event.target as IDBRequest<IDBCursorWithValue | null>).result;
    if (!cursor) return;
    const value: unknown = cursor.value;
    if (isStoredImport(value)) entries.push({ key: cursor.key, file: { filename: value.filename, bytes: value.bytes } });
    else cursor.delete();
    cursor.continue();
  };
  await done(transaction);
  db.close();
  return entries;
}

export async function saveImports(files: StoredImport[]) {
  const db = await openDb();
  const transaction = db.transaction(IMPORTS, "readwrite");
  for (const { filename, bytes } of files) transaction.objectStore(IMPORTS).add({ version: VERSION, filename, bytes });
  await done(transaction);
  db.close();
}

/** Deletes an import whose file no longer parses. */
export async function deleteImport(key: IDBValidKey) {
  const db = await openDb();
  const transaction = db.transaction(IMPORTS, "readwrite");
  transaction.objectStore(IMPORTS).delete(key);
  await done(transaction);
  db.close();
}
