# The emptied prime tower position list (undefined behaviour not ported)

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The per-plate tower positions `wipe_tower_x` / `wipe_tower_y` can be shorter than the plate list; upstream's own comment names the cases (`PartPlate.cpp:6774-6776`). Deleting a plate can then empty them, and creating a plate afterwards reads the first entry of the empty list. That read is undefined behaviour, which the port does not reproduce. This note extends "Prime tower positions on plate edits" in `web-plater-move-rotate-arrange.md`.

## Upstream
- **`PartPlateList::delete_plate`** erases the deleted plate's entry when the list reaches it (`PartPlate.cpp:6777-6780`). A project storing one position for several plates, such as `custom-gcode-overview/two-plates.3mf` with `wipe_tower_x` ["15"], loses its only entry when plate 0 is deleted.
- **`create_plate` → `set_default_wipe_tower_pos_for_plate`** first resizes both lists to the plate count, filling with `values.front()` (`:6344-6345`). On an empty vector, `front()` is undefined behaviour.
- **Plates are created** by "Add plate" and by an arrange that needs more beds (`postprocess_bed_index_for_selected`, `:7833-7879`).

## Ares
- **The session panics today.** Both `add_plate` and an arrange's `place_new_towers` index the empty list.
- **Explicit refusal instead.** Before any change, the session returns `InvalidInput`, naming the hole, wherever it would create a plate while either list is empty:
  - `add_plate` checks before it creates the plate;
  - an "arrange all" checks after the kernel and before `finalize` touches the plates, and only when an arranged item would land on a new plate (the same stepping past locked plates as `create_plates_for`). It releases any print-sequence locks it took (A5) before returning.
- **Not covered:** a plate's own arrange never creates plates. Reading an emptied list through `get_at` (the stored tower of a shown plate, `PartPlate.cpp:4068-4069`) is a sibling read of the same hole and stays out of this note.

## RED (session-only, `plater_tower_positions.rs`)
1. **`add_plate`:** load `two-plates.3mf`, delete plate 0, then add a plate. `add_plate` returns `InvalidInput`, and the session keeps one plate.
2. **"Arrange all":** the same project with both cubes scaled to 150 x 150 mm, so no two share a plate. Delete plate 0 (its cube goes to the unprintable area), then "arrange all". The arrange returns `InvalidInput`; every instance keeps its transform, and the session keeps one plate.
- **Before GREEN:** both cases panic (index out of bounds) in `place_new_towers`.

## GREEN
- **`check_positions_for_new_plate`** (`plater/arrange_job/wipe_tower.rs`) returns the `InvalidInput` when either list is empty.
- **`add_plate`** calls it before creating the plate.
- **"Arrange all"** calls it after the kernel only when `PlateList::needs_new_plate` (the stepping of `create_plates_for` without creating anything) finds an item past the plate list. Its temporary print-sequence locks are released before it returns.
