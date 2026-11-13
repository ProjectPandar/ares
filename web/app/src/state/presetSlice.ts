import type { ConfigOverrides, OptionMap, OptionValue, ParsedPresetBundle, ProfileSliceOptions, Scene, SliceResult, SlicingStatus } from "@ares-slicer/wasm";
import { extruderOverrides } from "./extruders";
import { FILAMENT_LIBRARY } from "./presets";
import { slicer } from "./slicer";
import { type ModelFile, type PresetSelection, useApp } from "./store";
import { decodeOptions } from "./typedSettings";

/** Installed vendors and imported presets of the engine catalog. */
export type CatalogSources = { vendors: readonly string[]; presetImports: readonly ParsedPresetBundle[] };

/** The `sliceWithProfiles` / `profileSettings` arguments of a complete selection. */
export async function profileRequest(scene: Scene, { printer, process, filaments, filamentProject }: PresetSelection, { vendors, presetImports }: CatalogSources, plate = scene.plates[0]?.id) {
  const { loadVendor } = await import("@ares-slicer/profiles");
  const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
  const catalog = JSON.stringify({ vendors: loaded, imports: presetImports });
  const options: ProfileSliceOptions = {
    plate,
    seed: 0,
    placement: scene.format === "stl" ? "center" : "none",
    overrides: filamentProject ? await overrideTokens(filamentProject) : undefined,
  };
  return { catalog, selection: { machine: printer, process: process!, filaments }, options };
}

/**
 * Sidebar edits as engine override tokens: text edits already hold the option's
 * `serialize()` text (the Field.cpp commit), typed values are encoded by the engine.
 */
export async function overrideTokens(overrides: ConfigOverrides): Promise<Record<string, string | string[]>> {
  const isText = (value: OptionValue): value is string | string[] =>
    typeof value === "string" || (Array.isArray(value) && value.every((element) => typeof element === "string"));
  const edits = Object.entries(overrides);
  const typed = edits.filter(([, value]) => !isText(value));
  const encoded = await slicer.encodeConfigValues(typed);
  return Object.fromEntries([
    ...edits.filter((edit): edit is [string, string | string[]] => isText(edit[1])),
    ...typed.map(([key], at) => [key, encoded[at]]),
  ]);
}

/**
 * Slices with the selected presets, like the OrcaSlicer CLI's `--load-settings` /
 * `--load-filaments`: plate `plate` of the edited 3MF over its project config, or an STL centered on
 * the bed as Plater::priv::load_model_objects places a model loaded on an empty
 * plate (Plater.cpp:7056-7058, 7111-7112, 7149-7160). Sidebar edits apply after
 * the presets, as an edit of the selected preset in a settings tab does.
 */
export async function sliceWithPresets(file: ModelFile, scene: Scene, selection: PresetSelection, overrides: ConfigOverrides, plate?: number, onStatus?: (status: SlicingStatus) => void): Promise<SliceResult> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, useApp.getState(), plate);
  // The plate combo's `curr_bed_type` is part of the project config the slice composes.
  // So are the dual-nozzle volume types and filament map.
  const project: ConfigOverrides = {
    ...(selection.bedType === undefined ? {} : { curr_bed_type: selection.bedType }),
    ...selection.filamentProject,
    ...extruderOverrides(selection, selection.filaments.length),
  };
  const request = { ...options, overrides: await overrideTokens({ ...overrides, ...project }) };
  const { plater } = useApp.getState();
  return plater
    ? plater.sliceWithProfiles(catalog, profiles, request, onStatus)
    : slicer.sliceWithProfiles(file.bytes, file.name, catalog, profiles, request);
}

/**
 * Values of the selected presets as the preset slice composes them (`PresetBundle::full_config`),
 * typed where decoded, for the process tab to show and edit.
 */
export async function presetValues(file: ModelFile, scene: Scene, selection: PresetSelection, sources: CatalogSources): Promise<OptionMap> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, sources);
  const settings = await slicer.profileSettings(file.bytes, file.name, catalog, profiles, options);
  return { ...settings, ...(await decodeOptions(useApp.getState().configDefs!, settings)) };
}

/**
 * The scene as the preset slice places it on the selected printer's bed: an STL centered like
 * one model loaded on an empty plate (Plater.cpp:7149-7160), with the composed config's bed.
 */
export async function placedScene(file: ModelFile, scene: Scene, selection: PresetSelection, sources: CatalogSources): Promise<Scene> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, sources);
  return slicer.placementScene(file.bytes, file.name, catalog, profiles, options);
}
