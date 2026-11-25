import type { FilamentSlotEdit, FilamentSlots, FilamentTarget, ProjectSettings, Scene } from "@ares-slicer/wasm";
import { filamentNames, stringList } from "../sidebar/options";
import { applyLayout } from "./plates";
import { useApp, type PresetSelection } from "./store";
import { takeSnapshot } from "./undoRedo";

export type FilamentSelection = FilamentTarget & { instance?: number };

export function filamentPalette(
  selection: PresetSelection | null,
  settings: ProjectSettings | null,
  project?: FilamentSlots,
) {
  if (project?.edited) return project.slots.map(({ colour }) => colour);
  return selection?.filamentProject?.filament_colour ?? stringList(settings?.filament, "filament_colour");
}

export function activeFilamentNames(selection: PresetSelection | null, scene: Scene | null) {
  if (scene?.filamentSlots?.edited) return scene.filamentSlots.slots.map(({ preset }) => preset);
  return selection?.filaments ?? filamentNames(scene?.settings ?? null);
}

export async function editFilamentSlots(edit: FilamentSlotEdit) {
  const { plater, currentPlate } = useApp.getState();
  if (!plater) return;
  takeSnapshot();
  const layout = await plater.editFilamentSlots(edit);
  if (useApp.getState().plater === plater) applyLayout(layout, currentPlate);
}

export function sameTarget(a: FilamentSelection, b: FilamentSelection) {
  return a.object === b.object && a.volume === b.volume && a.instance === b.instance;
}

export function chooseFilamentTarget(target: FilamentSelection, mode: "select" | "toggle" | "context") {
  const previous = useApp.getState().filamentSelection;
  const index = previous.findIndex((item) => sameTarget(item, target));
  if (mode === "context" && index !== -1) return;
  const targets =
    mode === "toggle" ? (index === -1 ? [...previous, target] : previous.filter((_, i) => i !== index)) : [target];
  const active = targets.at(-1);
  useApp.setState({
    filamentSelection: targets,
    selectedObject: active?.object ?? null,
    selectedInstance: active?.instance ?? null,
  });
}

export async function assignFilament(
  choice: { targets: readonly FilamentTarget[] } | { cell: FilamentTarget },
  slot: number,
) {
  const { plater, presetSelection, scene, currentPlate } = useApp.getState();
  if (!plater) return;
  const count = activeFilamentNames(presetSelection, scene).length;
  takeSnapshot();
  const layout = await ("cell" in choice
    ? plater.setFilamentCell(choice.cell, slot, count)
    : plater.setFilament(choice.targets, slot, count));
  if (useApp.getState().plater === plater) applyLayout(layout, currentPlate);
}
