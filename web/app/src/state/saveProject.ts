import { download } from "./actions";
import { presetCatalog } from "./presetCatalog";
import { overrideTokens, profileSliceRequest } from "./presetSlice";
import { FROM_PROJECT } from "./projectPresets";
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

/**
 * The current scene as saved-project bytes (Plater::save_project) and the project file stem, or
 * undefined when the project changed meanwhile.
 */
export async function projectBytes(): Promise<{ bytes: Uint8Array<ArrayBuffer>; stem: string } | undefined> {
  const opening = stlSessionOpened();
  const selectedFile = useApp.getState().file;
  await opening;
  const state = useApp.getState();
  const { file, scene, overrides, plater, presetSelection } = state;
  if (file !== selectedFile || saveProjectBlocker(state)) return;
  const stillOpen = () =>
    useApp.getState().file === file && useApp.getState().plater === plater && opening === stlSessionOpened();
  let bytes: Uint8Array<ArrayBuffer>;
  if (presetSelection) {
    const { profiles, request } = await profileSliceRequest(scene!, presetSelection, overrides);
    if (!stillOpen()) return;
    bytes = await plater!.saveProjectWithProfiles(await presetCatalog(), profiles, request);
  } else {
    const tokens = await overrideTokens(overrides);
    if (!stillOpen()) return;
    bytes =
      file!.kind === "stl-batch"
        ? await plater!.saveProject(tokens)
        : await plater!.saveProjectWithProfiles(await presetCatalog(), FROM_PROJECT, {
            seed: 0,
            placement: "none",
            overrides: tokens,
          });
  }
  if (!stillOpen()) return undefined;
  const stem = file!.name.replace(/\.(3mf|stl)$/i, "").replace(/[/\\\0]/g, "_");
  return { bytes, stem: stem || "Untitled" };
}

/** Browser equivalent of Plater::save_project: both menu commands download the current scene. */
export async function saveProject() {
  const file = useApp.getState().file;
  useApp.setState({ saveError: undefined });
  try {
    const project = await projectBytes();
    if (project) download(project.bytes, `${project.stem}.3mf`, "model/3mf");
  } catch (error) {
    if (useApp.getState().file === file) useApp.setState({ saveError: (error as Error).message });
  }
}
