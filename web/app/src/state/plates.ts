import type { PlaterLayout } from "@ares-slicer/wasm";
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
  if (sliced) setSliceResult(sliced, sliced.elapsedMs);
  else useApp.setState({ slice: { status: "idle" } });
}

/** The scene with an edit's instance matrices and plates; every slice result is stale. */
function applyLayout(layout: PlaterLayout, currentPlate: number) {
  useApp.setState(({ scene, slice }) => ({
    scene: {
      ...scene!,
      objects: scene!.objects.map((object, at) => ({ ...object, instances: layout.objects[at].instances })),
      plates: layout.plates,
    },
    currentPlate,
    ...invalidatedSlices(slice),
  }));
}

/** "Add plate" (Plater::priv::on_action_add_plate, Plater.cpp:10123-10136): appends a plate and selects it. */
export async function addPlate() {
  const layout = await useApp.getState().plater!.addPlate();
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
