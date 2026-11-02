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
- **Fixture:** `byobject-perobject-multi-copy-brim/combined-brim.3mf` (by object, `extruder_clearance_radius` 47, `nozzle_height` 2.5, `max_layer_height` 0.36, 10 x 10 x 4 mm cubes, all taller than `nozzle_height`, so no part is short). Edits:
  - `skirt_type` per object, `skirt_loops` 1, `skirt_distance` 2, `skirt_height` 10, `initial_layer_line_width` 0.5.
  - The skirt is then taller than the nozzle (10 × 0.36 > 2.5), so `o = 2 + 0.5 = 2.5` exactly, and the clearance is 52.
- **Cases** in `plater_arrange/object_skirt.rs`:
  1. "Arrange all" places the parts like Native's arrange of the fixture with `extruder_clearance_radius` 52. Native at 47 places them differently, which the case asserts so it discriminates.
  2. **Session-only, retargeted in RED:** the plate's own arrange of plate 0 equals the same arrange with a combined skirt (same `get_real_skirt_dist`, no offset) at radius 52. Native `--slice` cannot be the oracle: its slice validation adds the offset (`Print.cpp:917, 928`) that its arrange never applies, so it rejects every per-object-skirt plate with -63, at radius 47 as at 52. Case 1 is the Native anchor for these parameters.
  3. **Session-only:** a second, empty plate selected as the current plate gives a print with no objects, so the offset is 0. The arrange matches the same project with `skirt_loops` 0.
  4. A by-layer project with the per-object skirt places the parts like Native, since the offset changes nothing outside a by-object arrange.
  5. **Session-only:** `select_plate` / `current_plate`, plus `delete_plate`'s move back.
- **Web E2E:** the existing plate-arrange and multi-plate specs keep passing with the selection sent to the session.
- **Expected before GREEN:**
  - Cases 1-4 are refused ("per-object skirts").
  - Case 5 fails on the `todo!()` skeleton of `select_plate` / `current_plate`.

## GREEN
- **One port, `print/object_skirt.rs`.**
  - The locals are `f32` like upstream's floats: the maximum nozzle diameter, the maximum layer height, the first-layer line width (`get_abs_value`: `ratio * value / 100`), the skirt width and the offset.
  - The `f64` config reads are promoted and then narrowed exactly where C++ does it.
  - `margin_height` is its default 0 at both call sites, so the tall test is `skirt_height * max_layer_height > nozzle_height - 0`.
  - Case 1's 2.5 holds only because 2 and 0.5 are exact.
- **The short test** is `is_all_objects_are_short` on scaled integers. Each object's `PrintObject::height()` is `(size_z * (1. / SCALING_FACTOR)) as coord_t` (`PrintObject.cpp:161`), from `raw_bounding_box`'s Z extent: the model parts through the first instance's matrix without its offset, each vertex a `float` (`Model.cpp:1761-1775`). It is compared with `nozzle_height / SCALING_FACTOR`.
- **The BBL multi-nozzle branch is ported literally, without an oracle.** `nozzle_diameter_for_filament`'s `filament_map` path is in place; no case reaches it, and none is invented.
- **The arrange's print** is the current plate's.
  - The settings are the session's `project.settings()`, as the rest of the arrange parameters read them.
  - The objects with an instance on the plate are in model order; the first one's `line_width` and `support_filament` are its own override, else the process value.
  - `is_BBL_printer` is the vendor type's `is_bbl_vendor`.
- **The validation** (`print/validate/sequential_clearance.rs`) now calls the same port with its resolved first object and `project_is_bbl`, which `validate_applied_project` now computes one step earlier. It keeps its own `short` computation, which is outside this slice.
- **The current plate** is `PlaterSession::current_plate`.
  - `select_plate` sets it; `delete_plate` moves it back (`PartPlate.cpp:6809-6815`); a plate's own arrange selects its plate.
  - The web calls `selectPlate` when it selects a plate and after "Add plate", which selects the new plate (`Plater::priv::on_action_add_plate`).
- **Case 5 retargeted.** The first sequence deleted plate 0 of a project storing a single tower position, then added a plate. That empties the position list, and `create_plate` then reads `values.front()` of an empty vector (`PartPlate.cpp:6344` after `:6777-6780`), which is undefined behaviour upstream and a panic in the session. Case 5 now adds a plate before deleting. The undefined behaviour itself is outside this slice and reported separately.
- **Case 3 discriminates.** With the arrange reading plate 0 instead of the current plate, it fails.
