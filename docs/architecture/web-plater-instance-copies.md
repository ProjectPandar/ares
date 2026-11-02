# Instance copies on the plater session ("Add instance" / "Remove instance")

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The plater's "+" and "−" add a copy of the selected object next to its last instance, or remove its last instance. The session has no instance count edit today; `web-plater-move-rotate-arrange.md` lists instance copies as deferred.

## Upstream boundary
- **`Plater::increase_instances(num = 1)`** (`Plater.cpp:19289-19331`). From the object's last instance it adds `num` instances. The `i`-th is placed at the last instance's offset + `(i·o, i·o, 0)`, `i` = 1..num, where `o = get_size_proportional_to_max_bed_size(0.05)`, that is 0.05 × the larger side of the build volume's 2D bounding box (`GLCanvas3D.cpp:6195-6199`). Each copy takes the last instance's scaling factor, rotation and mirror.
  - Each copy is made by `ModelObject::add_instance(offset, scaling, rotation, mirror)` (`Model.cpp:1573-1581`). That calls `add_instance()`, a default `new ModelInstance(this)` (`:1554-1563`), and then sets four components only: `set_offset`, `set_scaling_factor` (`Geometry.cpp:528-535`), `set_rotation` (`:491-496`) and `set_mirror` (`:565-587`).
  - The four values are read from the last instance through `get_offset`, `get_scaling_factor` (`:513-517`), `get_rotation` (`:471-474`, Euler angles extracted from `extract_rotation_scale`) and `get_mirror` (`:550-554`).
  - Every other field takes `ModelInstance`'s default, not the last instance's: printable, the assemble transform, and the loaded and identify ids.
  - Each copy is then registered with the plate it lands on: `PartPlateList::notify_instance_update(obj, i)` (`PartPlate.cpp:7261`).
  - `SUPPORT_AUTO_CENTER` is not defined, so there is no arrange afterwards.
- **`Plater::decrease_instances(num = 1)`** (`Plater.cpp:19333-19364`): each removal is `notify_instance_removed` (`PartPlate.cpp:7396`) and then `delete_last_instance` (`Model.cpp:1591-1594`).
- **When they apply** (`Plater.cpp:15097-15118`):
  - "+" needs a selected object whose instances are all printable;
  - "−" needs one with more than one instance, so "−" never removes the object.
- **Where the GUI offers them:** the object menu's "Add instance" (shortcut "+") and "Remove instance" ("−") (`GUI_Factories.cpp:2310-2315`), and the toolbar's "more" / "fewer" (`GLCanvas3D.cpp:7590-7608`).
- **Export.** A copy goes into the GUI's project export like any instance. Its `identify_id` is `inst->id().id` (`bbs_3mf.cpp:8582-8605`), a run-time object counter, since a copy has no loaded id.

## Ares destination
- **`PlaterSession::increase_instances(object, num)` / `decrease_instances(object, num)`.** `InvalidInput` where upstream's `can_*` is false: no such object, a non-printable instance for "+", one instance for "−".
- **The project gains and loses instances.**
  - A copy is a new default instance (printable, no loaded id), not a duplicate of the last one.
  - Its transformation is composed literally. Start from the identity, then apply `set_offset`, `set_scaling_factor`, `set_rotation` and `set_mirror` with the last instance's `get_*` values. The rotation goes through `get_rotation`'s Euler extraction and `set_rotation`; the matrix is never copied.
  - `plater/transformation.rs` gains `get_scaling_factor` / `set_scaling_factor` and `get_mirror` / `set_mirror` beside its `get_rotation` / `set_rotation`.
  - Its imported copy, which slicing reads, is derived from that transformation as `set_instance_transform` derives the others'.
  - Its `instance_id` is its index in the object.
  - Its identify id is a deterministic stand-in for upstream's run-time counter: one past the largest identify id in the project, in creation order.
- **`PlaterEdits`** carries each object's instance list, so slicing from the session and the project export include the copies.
- **wasm:** `increaseInstances` / `decreaseInstances`.
- **Web:** the object menu gains "Add instance" and "Remove instance" with upstream's enabling rules, plus the "+" / "−" keys on the selection.

## Included and deferred
- **Included:** "+" and "−" for any count, plate registration, slicing and export of the copies, and the menu and keys.
- **Deferred:**
  - "Set number of instances" (a dialog over the same two calls);
  - "Fill bed with copies" (`fill_bed_with_copies`), which needs the arrange's fill mode;
  - "Set instance to object";
  - the toolbar buttons, since the web has no plater toolbar;
  - the undo snapshot.

## Prerequisite (move-only)
`crates/ares-cli/tests/orca_parity/plater_session.rs` is at 395 LOC. Before the new `plater_instances.rs` child is registered there, a move-only commit adds the group `plater_transform.rs`. It holds the scale, mirror and rotation-pivot registrations and the rotate case with its helpers, moved byte-identical. The group re-exports the parent's helpers, so the child files do not change.

## Oracle and RED
- **The session-edit Native oracle**, as the move and rotate cases use it (`assert_plate_like_native`): the session's edits are written into the project the way the GUI exports them, Native slices that export, and the session's slice must match. The test-side export gains the copies' build items and plate entries.
- **Fixture:** `orca_cli_ender3.3mf`, one 10 mm cube at (110, 110) on a 220 mm bed, so `o` = 11.
- **Cases** in `plater_instances.rs`:
  1. Two copies: they sit at (121, 121) and (132, 132) with the cube's rotation, scale and mirror; plate 1 holds all three; the plate slices like Native.
  2. Two copies, then one removed: the copy at (121, 121) remains, and the plate slices like Native.
  3. The cube rotated 30° about Z and mirrored on X before copying. `extract_rotation_scale` keeps the flip on X, which leaves the largest trace, so the copy's rotation is (0, 0, 30°), its scaling factor (1, 1, 1) and its mirror (−1, 1, 1). Composed, its linear part is the cube's, within 1e-9 of the file's 9-digit cosines. That is a literal derived from `Geometry.cpp`: no CLI path calls `add_instance(offset, scaling, rotation, mirror)` (its only `add_instance` is the assemble path's empty one, `OrcaSlicer.cpp:4925`), so Native cannot witness the composition. The plate then slices like Native.
  4. **Session-only:** "−" on a single instance and "+" on an object with a non-printable instance return `InvalidInput`.
- **Web E2E** `plateInstances.spec.ts`: "Add instance" twice from the object menu, then "Slice plate", compared with the runtime Native oracle for the exported project.
- **Expected before GREEN:** cases 1-4 fail on the `todo!()` skeleton of `increase_instances` / `decrease_instances`.
