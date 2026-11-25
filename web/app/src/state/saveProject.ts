import { download } from "./actions";
import { overrideTokens, profileSliceRequest } from "./presetSlice";
import { catalogJson, FROM_PROJECT } from "./projectPresets";
import { presetSliceBlocker } from "./presets";
import { stlSessionOpened } from "./stlSession";
import { useApp } from "./store";

export function saveProjectBlocker({
  file,
  scene,
  presetSelection,
  plater,
  loadingFile,
  arranging,
  slice,
}: ReturnType<typeof useApp.getState>) {
  if (loadingFile || arranging || slice.status === "slicing") return "Wait for the current plater operation to finish.";
  if (!file || !scene || !scene.objects.length) return "Open or import a model to save a project.";
  if (
    scene.format === "stl" &&
    file.kind === "single" &&
    (!presetSelection || presetSliceBlocker(presetSelection, scene))
  ) {
    return "Select a printer, process, and filament before saving this STL as a project.";
  }
  if (!plater) return "Wait for the project to finish opening.";
  return undefined;
}

/** Browser equivalent of Plater::save_project: both menu commands download the current scene. */
export async function saveProject() {
  const opening = stlSessionOpened();
  const selectedFile = useApp.getState().file;
  await opening;
  const state = useApp.getState();
  const { file, scene, overrides, plater, presetSelection } = state;
  if (file !== selectedFile || saveProjectBlocker(state)) return;
  const stillOpen = () =>
    useApp.getState().file === file && useApp.getState().plater === plater && opening === stlSessionOpened();
  useApp.setState({ saveError: undefined });
  try {
    let bytes: Uint8Array<ArrayBuffer>;
    if (presetSelection) {
      const { catalog, profiles, request } = await profileSliceRequest(scene!, presetSelection, overrides);
      if (!stillOpen()) return;
      bytes = await plater!.saveProjectWithProfiles(catalog, profiles, request);
    } else {
      const tokens = await overrideTokens(overrides);
      const catalog = file!.kind === "stl-batch" ? null : await catalogJson(state);
      if (!stillOpen()) return;
      bytes =
        catalog === null
          ? await plater!.saveProject(tokens)
          : await plater!.saveProjectWithProfiles(catalog, FROM_PROJECT, {
              seed: 0,
              placement: "none",
              overrides: tokens,
            });
    }
    if (!stillOpen()) return;
    const stem = file!.name.replace(/\.(3mf|stl)$/i, "").replace(/[/\\\0]/g, "_");
    download(bytes, `${stem || "Untitled"}.3mf`, "model/3mf");
  } catch (error) {
    if (stillOpen()) useApp.setState({ saveError: (error as Error).message });
  }
}
