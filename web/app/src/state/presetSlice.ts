import type {
  ConfigOverrides,
  OptionMap,
  OptionValue,
  ParsedPresetBundle,
  ProfileSliceOptions,
  Scene,
  SliceResult,
  SlicingStatus,
} from "@ares-slicer/wasm";
import { extruderOverrides } from "./extruders";
import { presetCatalog } from "./presetCatalog";
import { slicer } from "./slicer";
import { type ModelFile, type PresetSelection, useApp } from "./store";
import { decodeOptions } from "./typedSettings";

/** Installed vendors and imported presets of the engine catalog. */
export type CatalogSources = { vendors: readonly string[]; presetImports: readonly ParsedPresetBundle[] };

/** The `sliceWithProfiles` / `profileSettings` arguments of a complete selection. */
export async function profileRequest(
  scene: Scene,
  { printer, process, filaments, filamentProject }: PresetSelection,
  plate = scene.plates[0]?.id,
  includeOverrides = true,
) {
  const options: ProfileSliceOptions = {
    plate,
    seed: 0,
    placement: scene.format === "stl" ? "center" : "none",
    overrides: includeOverrides && filamentProject ? await overrideTokens(filamentProject) : undefined,
  };
  return { selection: { machine: printer, process: process!, filaments }, options };
}

/**
 * Sidebar edits as engine override tokens: text edits already hold the option's
 * `serialize()` text (the Field.cpp commit), typed values are encoded by the engine.
 */
export async function overrideTokens(overrides: ConfigOverrides): Promise<Record<string, string | string[]>> {
  const edits = Object.entries(overrides);
  if (edits.length === 0) return {};
  const isText = (value: OptionValue): value is string | string[] =>
    typeof value === "string" || (Array.isArray(value) && value.every((element) => typeof element === "string"));
  const typed = edits.filter(([, value]) => !isText(value));
  const encoded = typed.length > 0 ? await slicer.encodeConfigValues(typed) : [];
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
export async function profileSliceRequest(
  scene: Scene,
  selection: PresetSelection,
  overrides: ConfigOverrides,
  plate?: number,
) {
  const { selection: profiles, options } = await profileRequest(scene, selection, plate, false);
  // The plate combo's `curr_bed_type` is part of the project config the slice composes.
  // So are the dual-nozzle volume types and filament map.
  const project: ConfigOverrides = {
    ...(selection.bedType === undefined ? {} : { curr_bed_type: selection.bedType }),
    ...selection.filamentProject,
    ...extruderOverrides(selection, selection.filaments.length),
  };
  return { profiles, request: { ...options, overrides: await overrideTokens({ ...overrides, ...project }) } };
}

export async function sliceWithPresets(
  file: ModelFile,
  scene: Scene,
  selection: PresetSelection,
  overrides: ConfigOverrides,
  plate?: number,
  onStatus?: (status: SlicingStatus) => void,
): Promise<SliceResult> {
  const { profiles, request } = await profileSliceRequest(scene, selection, overrides, plate);
  const catalog = await presetCatalog();
  const { plater } = useApp.getState();
  if (file.kind === "stl-batch") return plater!.sliceWithProfiles(catalog, profiles, request, onStatus);
  return plater
    ? plater.sliceWithProfiles(catalog, profiles, request, onStatus)
    : catalog.sliceWithProfiles(file.bytes, file.name, profiles, request);
}

/**
 * Values of the selected presets as the preset slice composes them (`PresetBundle::full_config`),
 * typed where decoded, for the process tab to show and edit.
 */
export async function presetValues(file: ModelFile, scene: Scene, selection: PresetSelection): Promise<OptionMap> {
  const { selection: profiles, options } = await profileRequest(scene, selection);
  const catalog = await presetCatalog();
  const settings =
    file.kind === "stl-batch"
      ? await useApp.getState().plater!.profileSettings(catalog, profiles, options)
      : await catalog.profileSettings(file.bytes, file.name, profiles, options);
  return { ...settings, ...(await decodeOptions(useApp.getState().configDefs!, settings)) };
}

/**
 * The scene as the preset slice places it on the selected printer's bed: an STL centered like
 * one model loaded on an empty plate (Plater.cpp:7149-7160), with the composed config's bed.
 */
export async function placedScene(file: ModelFile, scene: Scene, selection: PresetSelection): Promise<Scene> {
  const { selection: profiles, options } = await profileRequest(scene, selection);
  if (file.kind === "stl-batch") return useApp.getState().plater!.scene();
  const catalog = await presetCatalog();
  return catalog.placementScene(file.bytes, file.name, profiles, options);
}
