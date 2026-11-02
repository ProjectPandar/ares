# "Delete All" on the plater session

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The Edit menu's "Delete All" (Ctrl+D) empties the model and resets what belongs to its objects; the plates stay. The session can delete one object (`web-plater-delete-object.md`) but has no "Delete All".

## Upstream boundary
- **The menu item** is the Edit menu's "Delete All", "Deletes all objects" (`MainFrame.cpp:3199-3201`).
  - It is enabled by `MainFrame::can_delete_all` (`:2128-2131`): the Prepare tab, and a model with objects.
  - Its shortcut is `Shortcut::DeleteAll`, Ctrl+D in the plater (`Shortcuts.cpp:80`), posting `EVT_GLTOOLBAR_DELETE_ALL` (`GLCanvas3D.cpp:4191`, bound at `Plater.cpp:8119`).
- **`Plater::priv::delete_all_objects_from_model`** (`Plater.cpp:10793-10821`):
  - drops the G-code preview and result, the sequential-clearance overlay and the background process, all UI or slicing state;
  - `partplate_list.clear()` (`PartPlate.cpp:6454-6499`, `delete_plates` false) empties every plate's instances and keeps the plates, so the current plate stays;
  - `model.clear_objects()` removes every object;
  - after `update()` / `object_list_changed()` as in "Delete", it resets `model.calib_pa_pattern` and clears `model.plates_custom_gcodes`, the per-plate custom G-code (layer pauses, colour changes) of every plate.
- **The plate menu's "Delete All"** (`GUI_Factories.cpp:1980-1985`, `remove_curr_plate_all`) is a different action: it selects the current plate's instances and deletes them through the selection, which needs per-instance deletion.

## Ares destination
- **`PlaterSession::delete_all_objects()`.**
  - `InvalidInput` without objects, where `can_delete_all` is false.
  - Otherwise every object is removed, every plate and the unprintable plate are emptied (plates, their locks and the current plate are kept), the arrange orders cleared, and `plates_custom_gcodes` cleared.
- **`PlaterEdits`** gains the cleared custom G-codes. `deleted_objects` already lists every loaded id. Slicing and the export then carry an empty model and no per-plate custom G-code.
- **wasm:** `deleteAllObjects`.
- **Web:**
  - Ctrl+D (⌘+D on macOS, as the app's other Ctrl shortcuts) on the Prepare page while the model has objects.
  - The shortcuts list gains the plater object shortcuts the app now implements, in upstream's wording: "Delete Selected", "Delete All", "Add instance" and "Remove instance". The earlier delete and instance slices left "Delete Selected" and the instance keys out of it.

## Included and deferred
- **Included:** the Edit menu action through its shortcut, its model, plate and custom G-code resets, and the shortcut list entries.
- **Deferred:**
  - the plate menu's "Delete All" (`remove_curr_plate_all`), which needs per-instance deletion;
  - an Edit menu, since the web has none;
  - the PA calibration pattern, since the session has none;
  - the undo snapshot.

## Oracle and RED
- **Native for the end state.** After "Delete All" the exported project has no objects. Native slicing a plate with no objects fails with the model read error "The supplied file couldn't be read because it's empty" (`Model.cpp:354-355`), which `empty_selected_plate.rs` already compares. The session's slice of the emptied plate must fail the same way.
- **Cases** in `plater_delete_all.rs`:
  1. `two-plates.3mf`, whose two plates carry custom G-code (colour changes, pauses, templates). "Delete All" leaves two empty plates, the current plate unchanged and no custom G-code. Plate 1's slice fails like Native's on the export (`NoSuitableObjects` with that message).
  2. **Session-only:** "Delete All" on an emptied session returns `InvalidInput`; a locked plate stays locked.
- **Web E2E** `plateDeleteAll.spec.ts`: Ctrl+D on `two-plates.3mf`. The object list shows two plates without objects, and the shortcuts list shows the four entries.
- **Expected before GREEN:** cases 1-2 fail on the `todo!()` skeleton of `delete_all_objects`.
