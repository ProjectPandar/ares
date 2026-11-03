# The plate menu's "Delete All" on the plater session

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The plate menu's "Delete All" ("Delete all objects on the current plate") removes what is on the current plate. It differs from the Edit menu's "Delete All", which empties the model (`web-plater-delete-all.md`). It needs per-instance deletion (`web-plater-delete-instance.md`), now ported.

## Upstream boundary
- **The menu item** (`GUI_Factories.cpp:1980-1987`, in `create_plate_menu`, `:1958`) is shown on a plate item's right click in the object list (`GUI_ObjectList.cpp:1699`). It is enabled by `!plate->get_objects().empty()`. `PartPlate::get_objects()` returns all of the model's objects (`PartPlate.hpp:477`), so the item is enabled whenever the model has an object.
- **`Plater::priv::remove_curr_plate_all` → `GLCanvas3D::remove_curr_plate_all` → `Selection::remove_curr_plate`** (`Plater.cpp:10704-10709`, `GLCanvas3D.cpp:3048-3052`, `Selection.cpp:507-525`).
  - It selects every volume of each instance the current plate contains or overlaps: `contain_instance || intersect_instance`, `get_volume_idxs_from_plate` (`Selection.cpp:2256`).
  - An empty selection returns without a change.
  - Otherwise it classifies the selection (`update_type`, `:2309`) and calls `erase()` (`:1893-1996`).
- **What `erase()` deletes.** Every instance is selected whole, so each branch reduces to one rule per object:
  - all of its instances selected: the object is deleted;
  - otherwise each selected instance is deleted.

  The branches are single full object, multiple full object, single full instance, multiple full instance, and mixed. The mixed branch turns a fully selected object into `itObject` and a partly selected one's instances into `itInstance` items; a one-instance object's volumes collapse back to `itObject`.
- **Deletion order.** `delete_from_model_and_list` walks the items in reverse `ItemForDelete` order: object index, then sub-index, type ignored (`GUI_ObjectList.hpp:86-91`; `GUI_ObjectList.cpp:4490-4558`). Objects and instances therefore go from the highest index down.
  - An object goes through `del_object`, the ported object delete: `notify_instance_removed(obj, -1)`.
  - An instance goes through `del_subobject_from_object`, the ported instance delete: no plate notified, pairs kept.

## Ares destination
- **`PlaterSession::delete_plate_objects()`** works on the current plate.
  - The selection: the instances whose pair the plate holds, or whose bounding box intersects the plate (the plate list's existing `intersects`).
  - For each touched object, from the highest index down: `delete_object` when every instance is selected, else `delete_instance` for each selected instance, from the highest index down.
  - Without instances it returns `Ok` and changes nothing.
- **wasm:** `deletePlateObjects`.
- **Web:** a plate row's right click opens a plate menu with "Delete All" ("Delete all objects on the current plate"). It is enabled while the model has objects, and acts on the plate it was opened on, after selecting it.

## Included and deferred
- **Included:** the plate menu's "Delete All".
- **Deferred:**
  - the plate menu's other items (Select All, arrange, rename, lock and delete plate, which the plate row's icons already offer);
  - volume-level selections, which this menu cannot make;
  - the undo snapshot.

## Oracle and RED
In `plater_plate_delete_all.rs`, the session-edit Native slice of the GUI export:
1. **Whole objects.** `two-plates.3mf` with plate 2 current: "Delete All" removes plate 2's object. Plate 1 keeps its object and slices like Native.
2. **Partial object.** `combined-brim.3mf` with object 2's instance 1 moved onto an added plate 2, and plate 2 current. Object 2 loses that instance (an instance delete). Plate 1 slices like Native.
3. **Whole multi-instance object.** `combined-brim.3mf` with plate 1 current: both objects go and the plates stay.
4. **Empty plate.** "Delete All" on an empty added plate changes nothing.

Web E2E `platePlateDeleteAll.spec.ts`: right-click plate 2 of `two-plates.3mf`, choose "Delete All", and plate 1 slices like Native.
