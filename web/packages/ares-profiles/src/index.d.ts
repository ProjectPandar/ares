/**
 * OrcaSlicer system presets (`resources/profiles`), one entry per vendor index
 * file, as loaded by `PresetBundle::load_vendor_configs_from_json`. Data only:
 * inheritance (`inherits`), compatibility and config composition belong to
 * `@ares-slicer/wasm`.
 */

/** OrcaSlicer commit the data was generated from. */
export declare const ORCA_SLICER_COMMIT: string;

export interface VendorSummary {
  /** Vendor index file stem, e.g. `"BBL"` for `BBL.json`. */
  id: string;
  name?: string;
  version?: string;
  description?: string;
}

/** Every vendor, sorted by `id`. Loading a vendor's presets is separate. */
export declare const vendors: readonly VendorSummary[];

/** `machine_model_list` / `process_list` / `filament_list` / `machine_list` entry. */
export interface VendorIndexEntry {
  name: string;
  sub_path: string;
}

/** Vendor index file (`<vendor>.json`), as in the OrcaSlicer source. */
export interface VendorIndex {
  name?: string;
  version?: string;
  description?: string;
  force_update?: string;
  url?: string;
  machine_model_list: VendorIndexEntry[];
  process_list: VendorIndexEntry[];
  filament_list: VendorIndexEntry[];
  machine_list: VendorIndexEntry[];
  [key: string]: unknown;
}

/** One preset or machine model file, as in the OrcaSlicer source. */
export interface ProfileJson {
  name: string;
  type?: string;
  inherits?: string;
  instantiation?: string;
  [key: string]: unknown;
}

export interface VendorProfiles {
  /** Vendor index file stem, the `PresetBundle::vendors` key (`BBL` for `BBL.json`). */
  id: string;
  index: VendorIndex;
  /** Every file listed in the index, keyed by its `sub_path`. */
  files: Record<string, ProfileJson>;
}

/**
 * Every vendor that has printer models, with only its `machine_model_list` files, as the
 * setup guide lists them (`GuideFrame::BuildProfileJson`). One chunk for all vendors.
 */
export declare function loadPrinterModels(): Promise<VendorProfiles[]>;

/** Loads one vendor's index and preset files; bundlers emit one chunk per vendor. */
export declare function loadVendor(id: string): Promise<VendorProfiles>;
