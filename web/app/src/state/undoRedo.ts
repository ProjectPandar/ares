/**
 * Undo and redo of plater edits (Plater::priv::take_snapshot / undo / redo / undo_redo_to,
 * Plater.cpp:15260-15381, 15406, over UndoRedo::Stack, slic3r/Utils/UndoRedo.cpp:937-1145).
 * Before an edit the plater's model and plates are saved with the selection; undo puts back the
 * previous saved state, first saving the shown one so redo can return to it, and a new edit drops
 * the states ahead. A new plater session (a loaded project) starts an empty stack
 * (`SnapshotType::ProjectSeparator`); the worker releases the old session's snapshots with it.
 */
import type { SceneObject } from "@ares-slicer/wasm";
import { applyLayout } from "./plates";
import { useApp } from "./store";

interface Snapshot {
  id: number;
  currentPlate: number;
  selectedObject: number | null;
  selectedInstance: number | null;
}

let snapshots: Snapshot[] = [];
/** The index of the shown state; `snapshots.length` while it is not saved (the topmost state). */
let active = 0;
/**
 * The objects of the loaded scene: an edit changes only instances and removes objects, so every
 * snapshot's objects are among them.
 */
let loadedObjects: readonly SceneObject[] = [];

useApp.subscribe(({ plater, scene }, previous) => {
  if (plater === previous.plater) return;
  snapshots = [];
  active = 0;
  loadedObjects = scene?.objects ?? [];
});

function capture(): Snapshot {
  const { plater, currentPlate, selectedObject, selectedInstance } = useApp.getState();
  return { id: plater!.takeSnapshot(), currentPlate, selectedObject, selectedInstance };
}

/** Saves the plater before an edit (`Plater::TakeSnapshot`). */
export function takeSnapshot() {
  const dropped = snapshots.splice(active);
  if (dropped.length) void useApp.getState().plater!.dropSnapshots(dropped.map(({ id }) => id));
  snapshots.push(capture());
  active = snapshots.length;
}

/** Undo and redo wait for a running arrange (`undo_redo_blocked_by_job`) and slice. */
function blocked() {
  const { plater, arranging, slice } = useApp.getState();
  return !plater || arranging || slice.status === "slicing";
}

export async function undo() {
  if (blocked() || active === 0) return;
  if (active === snapshots.length) snapshots.push(capture());
  await load(--active);
}

export async function redo() {
  if (blocked() || active + 1 >= snapshots.length) return;
  await load(++active);
}

async function load(index: number) {
  const { id, currentPlate, selectedObject, selectedInstance } = snapshots[index];
  const layout = await useApp.getState().plater!.loadSnapshot(id);
  useApp.setState({ selectedObject, selectedInstance });
  applyLayout(layout, currentPlate, loadedObjects);
}
