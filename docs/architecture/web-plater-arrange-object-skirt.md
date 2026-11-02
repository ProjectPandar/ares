# Per-object skirts in the arrange (A8)

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. A per-object skirt widens the clearance a by-object arrange keeps between parts. The plater session refuses any project with a per-object skirt today ("arranging around per-object skirts is not supported yet", `params::object_skirt_offset`). That is the arrange's last refusal. This slice ports the offset, the current plate it is read from, and lifts the refusal.

## Upstream boundary
- **`init_arrange_params`** (`ArrangeJob.cpp:989-1001`): it takes `Print::object_skirt_offset()` from `get_current_fff_print()`, the current plate's print. It sets `clearance_radius = extruder_clearance_radius + offset * 2` and `object_skirt_offset = offset`.
- **The readers** are all inside `params.is_seq_print`, so a by-layer arrange never feels the offset:
  - `update_arrange_params`'s bed shrink (`Arrange.cpp:156-159`);
  - `update_selected_items_inflation`'s minimum distance (`:168-175`: twice the offset when every part is short, else the clearance radius);
  - `update_unselected_items_inflation`'s exclusion gap (`:216-221`).
  - Ares already ports all three over `ArrangeParams`.
- **`Print::object_skirt_offset`** (`Print.cpp:5796-5817`):
  - zero without a per-object skirt, or with no objects;
  - otherwise one of three cases: all objects short (`skirt_distance` + skirt width), draft shield or a skirt taller than the nozzle (`skirt_distance` + the first-layer line width), or a skirt wider than half the clearance radius;
  - zero beyond those.
- **Its helpers:**
  - `Print::skirt_flow` (`:3084-3104`): the first-layer line width, else the first object's `line_width`, on the nozzle of the first object's support filament;
  - `nozzle_diameter_for_filament` (`PrintConfig.cpp:148-155`): a Bambu Lab printer with several nozzles maps the filament through `filament_map`;
  - `Print::is_all_objects_are_short` (`Print.hpp:1379-1381`): every object lower than `nozzle_height`.
- **The current plate.**
  - `PartPlateList::select_plate` sets `m_current_plate` (`PartPlate.cpp:6958-7007`).
  - The plater selects plate 0 after loading a project (`Plater.cpp:10222, 10229`).
  - `delete_plate` moves the current plate back when it deletes the current plate or one before it (`PartPlate.cpp:6809-6815`).
  - The plate's arrange icon selects its plate before arranging.
- **The CLI** never sets `object_skirt_offset` (`OrcaSlicer.cpp` has no use of it), so its arrange reads only `extruder_clearance_radius`.

## Ares destination
- **One port of `Print::object_skirt_offset`.**
  - It goes to `crates/ares-core/src/print/object_skirt.rs`, with `skirt_flow`, `nozzle_diameter_for_filament` and the short-objects test. Its inputs are the full settings, the first object's `line_width` and `support_filament`, whether every object is short, and `is_bbl_printer`.
  - The sequential-clearance validation already has a 2.4.2-cited copy (`print/validate/sequential_clearance/skirt.rs`), which becomes a caller of it.
  - That brings the validation to 2.5's nozzle resolution: a Bambu Lab multi-nozzle printer's filament goes through `filament_map`. No case covers that branch, which no Native oracle reaches through the validation either.
- **The session's current plate:**
  - `PlaterSession::select_plate(plate)` and `current_plate()` (public): 0 after loading, moved back by `delete_plate` as upstream does, and `InvalidInput` for a missing plate.
  - `ArrangeScope::CurrentPlate(n)` selects plate `n` first, as the icon does.
- **The arrange's print** is the current plate's:
  - the objects with an instance on it, in model order;
  - their heights for the short test, `PrintObject::height`, each object's transformed height;
  - `is_bbl_printer` from the session's vendor type (`is_bbl_vendor`).
  - `params::object_skirt_offset` and its refusal are replaced.
- **Web:** wasm `selectPlate`. The web's plate selection calls it, as it calls the session for every other plate edit.

## Included and deferred
- **Included:** the offset in both scopes, the current plate, the shared port and the refusal lifted. After this, the arrange refuses nothing.
- **Deferred:**
  - the selection snapshot (`take_snapshot("select partplate!")`);
  - the bed-position update, which is scene state.

## Oracle and RED
- **Derived Native oracle.** For tall parts in a by-object print, every reader sees only `clearance_radius`. So the GUI arrange of a project with offset `o` places parts exactly as Native's arrange of the same project with `extruder_clearance_radius` raised by `2·o`; Native ignores the offset.
- **Fixture:** `byobject-perobject-multi-copy-brim/combined-brim.3mf` (by object, `extruder_clearance_radius` 47, `nozzle_height` 2.5, `max_layer_height` 0.36, 10 mm cubes, so no part is short). Edits:
  - `skirt_type` per object, `skirt_loops` 1, `skirt_distance` 2, `skirt_height` 10, `initial_layer_line_width` 0.5.
  - The skirt is then taller than the nozzle (10 × 0.36 > 2.5), so `o = 2 + 0.5 = 2.5` exactly, and the clearance is 52.
- **Cases** in `plater_arrange/object_skirt.rs`:
  1. "Arrange all" places the parts like Native's arrange of the fixture with `extruder_clearance_radius` 52. Native at 47 places them differently, which the case asserts so it discriminates.
  2. The plate's own arrange of plate 0, against Native `--slice 1` at 52.
  3. **Session-only:** a second, empty plate selected as the current plate gives a print with no objects, so the offset is 0. The arrange matches the same project with `skirt_loops` 0.
  4. A by-layer project with the per-object skirt places the parts like Native, since the offset changes nothing outside a by-object arrange.
  5. **Session-only:** `select_plate` / `current_plate`, plus `delete_plate`'s move back.
- **Web E2E:** the existing plate-arrange and multi-plate specs keep passing with the selection sent to the session.
- **Expected before GREEN:**
  - Cases 1-4 are refused ("per-object skirts").
  - Case 5 fails on the `todo!()` skeleton of `select_plate` / `current_plate`.
