import type { PlaterLayout, ScaleEdit } from "@ares-slicer/wasm";
import type { InstanceRef } from "../viewport/instanceDrag";
import { setSliceResult } from "./actions";
import { invalidatedSlices, useApp } from "./store";

/** `PartPlateList::MAX_PLATES_COUNT` (PartPlate.hpp:36, 682): "Add plate" stops there (Plater.cpp:11413-11416). */
export const MAX_PLATES = 36;

/**
 * `PartPlateList::select_plate` (PartPlate.cpp:4761): the plate becomes current and the preview
 * shows its latest slice (`update_slicing_context_to_current_plate`, Plater.cpp:18185-18189).
 */
export async function selectPlate(index: number) {
  const { plater, plateSlices, slice } = useApp.getState();
  if (slice.status === "slicing") return;
  const sliced = plateSlices[index];
  if (sliced?.toolpaths) await plater!.preview(index + 1);
  useApp.setState({ currentPlate: index });
  // The worker runs requests in order, so an arrange after this reads the new current plate.
  await plater!.selectPlate(index);
  if (sliced) setSliceResult(sliced, sliced.elapsedMs);
  else useApp.setState({ slice: { status: "idle" } });
}

/**
 * The scene with an edit's instance matrices and plates, its objects in the layout's model order
 * (an arrange sorts them); every slice result is stale.
 */
export function applyLayout(layout: PlaterLayout, currentPlate: number) {
  useApp.setState(({ scene, slice }) => ({
    scene: {
      ...scene!,
      objects: layout.objects.map(({ id, instances }) => ({ ...scene!.objects.find((object) => object.id === id)!, instances })),
      plates: layout.plates,
      wipeTower: layout.wipeTower,
    },
    currentPlate,
    ...invalidatedSlices(slice),
  }));
}

/** "Add plate" (Plater::priv::on_action_add_plate, Plater.cpp:10123-10136): appends a plate and selects it. */
export async function addPlate() {
  const { plater } = useApp.getState();
  const added = await plater!.addPlate();
  const layout = await plater!.selectPlate(added.plates.length - 1);
  applyLayout(layout, layout.plates.length - 1);
}

/**
 * "Delete plate" (Plater::delete_plate, Plater.cpp:18149-18171): later plates move up; a
 * current plate at or after the deleted one moves up with them (PartPlate.cpp:4619-4625).
 */
export async function deletePlate(index: number) {
  const { plater, currentPlate } = useApp.getState();
  const layout = await plater!.deletePlate(index);
  applyLayout(layout, currentPlate >= index && currentPlate > 0 ? currentPlate - 1 : currentPlate);
}

/** The plate's lock icon (Plater.cpp:23414-23419): toggles the plate's lock. */
export async function lockPlate(index: number, locked: boolean) {
  const { plater, currentPlate } = useApp.getState();
  const layout = await plater!.lockPlate(index, locked);
  applyLayout(layout, currentPlate);
}

type Vec3 = [number, number, number];
type Step = { kind: "move" | "rotate"; target: InstanceRef; vector: Vec3 };
const steps: Step[] = [];
let editing = false;

/**
 * Canvas drags and gizmo steps run one at a time in the worker; steps of one kind on one instance
 * that arrive meanwhile are summed and sent as one (gizmo drags turn about a single axis).
 */
async function enqueue(step: Step) {
  const last = steps.at(-1);
  if (last && last.kind === step.kind && last.target.object === step.target.object && last.target.instance === step.target.instance) {
    last.vector = last.vector.map((value, axis) => value + step.vector[axis]) as Vec3;
  }
  else steps.push(step);
  if (editing) return;
  editing = true;
  try {
    for (let next = steps.shift(); next; next = steps.shift()) {
      const { plater, currentPlate } = useApp.getState();
      const { object, instance } = next.target;
      const layout = next.kind === "move" ? await plater!.translate(object, instance, next.vector) : await plater!.rotate(object, instance, next.vector);
      applyLayout(layout, currentPlate);
    }
  } finally {
    editing = false;
  }
}

/** A canvas drag or move gizmo step (Selection::translate, then GLCanvas3D::do_move). */
export function moveInstance(target: InstanceRef, displacement: Vec3) {
  return enqueue({ kind: "move", target, vector: displacement });
}

/** A rotate gizmo step about world axes, in radians (Selection::rotate, then GLCanvas3D::do_rotate). */
export function rotateInstance(target: InstanceRef, radians: Vec3) {
  return enqueue({ kind: "rotate", target, vector: radians });
}

/** A Position field edit in world coordinates (GizmoObjectManipulation::change_position_value). */
export async function setInstancePosition(target: InstanceRef, axis: 0 | 1 | 2, value: number) {
  const { plater, currentPlate } = useApp.getState();
  applyLayout(await plater!.setPosition(target.object, target.instance, axis, value), currentPlate);
}

/** The object menu's "Mirror" along an axis (Plater::mirror). */
export async function mirrorInstance(target: InstanceRef, axis: 0 | 1 | 2) {
  const { plater, currentPlate } = useApp.getState();
  applyLayout(await plater!.mirror(target.object, target.instance, axis), currentPlate);
}

/** A Scale field edit in percent (GizmoObjectManipulation::change_scale_value). */
export async function setInstanceScale(target: InstanceRef, edit: ScaleEdit) {
  const { plater, currentPlate } = useApp.getState();
  applyLayout(await plater!.setScale(target.object, target.instance, edit), currentPlate);
}

/** A Size field edit in millimetres (GizmoObjectManipulation::change_size_value). */
export async function setInstanceSize(target: InstanceRef, edit: ScaleEdit) {
  const { plater, currentPlate } = useApp.getState();
  applyLayout(await plater!.setSize(target.object, target.instance, edit), currentPlate);
}

/** An absolute rotation field edit in degrees (GizmoObjectManipulation::change_absolute_rotation_value). */
export async function setInstanceRotation(target: InstanceRef, axis: 0 | 1 | 2, degrees: number) {
  const { plater, currentPlate } = useApp.getState();
  applyLayout(await plater!.setAbsoluteRotation(target.object, target.instance, axis, degrees), currentPlate);
}
