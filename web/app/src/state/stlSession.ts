import type { PlaterSession, Scene } from "@ares-slicer/wasm";
import { loadArrangeSettings } from "./arrange";
import { installVendors } from "./presets";
import { type CatalogSources, profileRequest } from "./presetSlice";
import { slicer } from "./slicer";
import { invalidatedSlices, type ModelFile, type PresetSelection, useApp } from "./store";

/**
 * An STL with complete presets becomes a plater session on the selected printer's bed, centered
 * like one model loaded on an empty plate (Plater.cpp:7149-7160), so it can be moved and sliced
 * per plate; the store keeps it as an STL scene without project settings. Returns the session's
 * scene, whose `settings` are the composed config.
 */
export function openStlSession(file: ModelFile, scene: Scene, selection: PresetSelection, sources: CatalogSources): Promise<Scene> {
  // Each canvas (Prepare, Preview) places the model when it mounts; the plater keeps the session
  // it holds for the same inputs, as switching tabs reloads nothing upstream.
  const held = openedSession;
  if (held && held.plater === useApp.getState().plater && held.file === file && held.selection === selection
    && held.sources.vendors === sources.vendors && held.sources.presetImports === sources.presetImports) {
    return Promise.resolve(held.placed);
  }
  const opening = openSession(file, scene, selection, sources);
  stlSessionOpening = opening.catch(() => undefined);
  return opening;
}

/**
 * The latest STL session opening. The GUI places a model before it can be sliced (Plater::load_files
 * runs on the UI thread); a slice waits for this, so it slices the session the plater ends with and
 * its result outlives the session that opens after it.
 */
let stlSessionOpening: Promise<unknown> = Promise.resolve();
/** The session the plater holds and the inputs it was opened for. */
let openedSession: { plater: PlaterSession; file: ModelFile; selection: PresetSelection; sources: CatalogSources; placed: Scene } | null = null;

export function stlSessionOpened(): Promise<unknown> {
  return stlSessionOpening;
}

async function openSession(file: ModelFile, scene: Scene, selection: PresetSelection, sources: CatalogSources): Promise<Scene> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, sources);
  const plater = await slicer.createPlaterSessionWithProfiles(file.bytes, file.name, catalog, profiles, options);
  await installVendors(plater);
  const placed = await plater.scene();
  if (useApp.getState().file !== file) {
    void plater.free();
    return placed;
  }
  const previous = useApp.getState().plater;
  useApp.setState(({ slice }) => ({ plater, scene: { ...placed, format: "stl", settings: null }, currentPlate: 0, arrange: null, ...invalidatedSlices(slice) }));
  openedSession = { plater, file, selection, sources, placed };
  await loadArrangeSettings(plater);
  void previous?.free();
  return placed;
}
