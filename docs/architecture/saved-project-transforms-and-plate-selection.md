# Saved-project transforms and CLI plate selection

Source: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Citations to `OrcaSlicer.cpp` refer to `src/OrcaSlicer.cpp`; other Orca paths are under `src/`
unless noted. The target is the CLI's use of the `libslic3r` importer, model transformations,
arrangement and print admission, not a second model or slicing pipeline.

## What runs before a plate is sliced

1. **Import object membership, not a slice view.** `OrcaSlicer.cpp:1573–1576,1758` passes the
   requested slice ID to `Model::read_from_file`. `libslic3r/Format/bbs_3mf.cpp:2202–2222` uses a
   valid positive ID to admit objects listed by that saved plate, retaining all instances of
   each admitted object. `0` imports all objects. An ID without matching importer plate data
   does not filter the import or produce the later invalid-ID diagnostic. Keep the plate grid,
   per-plate settings, tower coordinates and saved instance identities separate from admission.
   Loaded-profile placement precedes CLI transforms (`OrcaSlicer.cpp:4418–4447,4602–4713`).
2. **Transform the whole imported model once.** `OrcaSlicer.cpp:4909,5051–5153` processes
   transform keys in first-occurrence order using the last scalar value (`libslic3r/Config.cpp`,
   `DynamicConfig::read_cli`). `--arrange 0` clears the arrangement decision, `1` forces it, and
   other values leave it unchanged; this branch does not pack anything. `--rotate` acts on
   volumes about Z (`:5069–5073`), X/Y and face placement follow the same ordered dispatch
   (`:5074–5141`), and `--scale` narrows the ratio to `float` before checking and applying it to
   volumes (`:5142–5153`). Apply requested orientation after this loop (`:5268–5288`). For `0`
   the model is the whole archive's admitted population; for positive `N` it is the population
   admitted for `N`, **not** just the instances listed on `N` and **not** every archived object.
3. **Check the signed slice ID, then arrange if needed.** `OrcaSlicer.cpp:5291–5296` rejects
   negative or out-of-range IDs after rotation, scale and orientation, but before packing.
   A bad scale therefore precedes this check; a bad slice ID precedes an arrange failure.
   `:5619–6100` packs the transformed model before any per-plate slice view: without duplication,
   `0` arranges globally (`:5687–5721`) and a positive ID arranges the current plate, considering
   membership/intersection and treating other locked instances separately (`:5679–5685,5815–5848`).
   Apply poses and updated membership back to the model (`:6021–6099`). `:5661–5665` starts the
   selected-plate tower position snapshot in the duplication branch; the X/Y reads are at
   `:5666–5671`. For selected-plate arrangement **without** duplication, `:5867–5871` reads the
   saved tower positions at `N - 1`; global arrangement uses per-plate inputs (`:5729–5812`).
   These are not tower coordinates of a rebased one-plate view. Explicit `ensure_on_bed`, if
   enabled, follows arrangement (`:6272–6281`).
4. **Select slice views from the resulting model.** `OrcaSlicer.cpp:6441–6445,6477–6528` slices
   every plate for `0` or one plate for `N`, with all-plates preflight before export. The initial
   print-volume classification precedes skip processing (`:6558–6579`). In `:6580–6673`, a saved
   identity match marks the instance non-printable and bypasses its subsequent checks; unmatched
   instances still undergo those checks. A plate with no inside instances initially is different
   from one whose inside instances were all skipped. Preserve the one-based plate in reports.

## Rust ownership and composition

Paths below are relative to the respective crate's `src/` directory. Names in
`project_slice/cli_transforms.rs` refer to the transform module being restored.

- **One parsed action set.** `ares-cli` `args.rs::SliceArgs::parse_transforms` and
  `main.rs::run_slice` must produce and pass one `CliProjectTransforms` through
  `slice_plates.rs::run` to `project_slice/cli_plates.rs::slice_cli_project_plates`, including
  when `--slice` dispatches before the single-output routes. Keep single-output `--plate N`
  and no-plate dispatch through `project_presets::slice`, `load_settings::slice`, and the
  Embedded project entry. Keep `CliPlatesSettings::{Embedded,Catalog,Files}` source resolution,
  installed resource lookup, warning and report ownership. The CLI owns files and status
  presentation, not transforms or mesh edits.
- **Finish importer admission before any CLI model edits.** In `ares-core`,
  `project_slice/state.rs::load_project_source` (Embedded) and
  `project_slice/profile_entry.rs::load_profile_slice` (Catalog/Files, via
  `load_with_profile_preparation`) return the admitted project and existing preparation context.
  Use `project/domain/import_scope.rs::Project::prepare_import_scope` for Embedded explicit
  saved-plate admission in the load prefix; it can discover membership without reporting an
  invalid slice ID. Keep the Catalog/Files positive ID flowing into
  `project/domain/profile_input/three_mf.rs::Project::from_3mf_with_profiles`: its existing
  `prepare_import_scope` before `shrink_to_new_bed_offset` is intentional. Removing the ID there
  changes the bounds used for placement and can also shift tower options. Do not move compatibility,
  settings validation, or upward-machine report snapshots behind model actions. No-action
  calls must continue to match the existing slice behavior. Non-CLI load and placement callers
  retain their current semantics.
- **Split the old transform call at the Native ID check.** In `slice_cli_project_plates` and the
  single-output preparation paths (`state.rs::prepare_project_source` and
  `profile_entry.rs::prepare_profile_slice`), use the same staged `CliProjectTransforms`
  implementation. The first stage operates on the already admitted `Project`, through
  `project/domain/auto_orient.rs::Project::{scale_cli_volumes,rotate_cli_volumes}` and
  `plater/auto_orient/project.rs::apply_cli_orientation`. Replace the CLI loaders' separate
  orientation dispatch; orientation must run once and must not perform late importer preparation.
  For `--slice`, `plater/initial_plate.rs::initialize_slice_plates` supplies the default plate if
  necessary, then `slice_cli_project_plates` checks the signed request against that plate list
  before any forced arrange. The second stage uses `plater/arrange_job.rs::arrange_cli_project`
  with `ArrangeScope::All` for `0` or `CurrentPlate` for `N`, reusing the arrange job, bed/exclusion
  inputs and tower position handling. No per-plate re-execution. Enumerate the all-plates output
  set from the **resulting** plate list, not a list captured before arrangement. The one-output
  route still slices one plate when the arrangement covered the whole project.
- **Project a view, never reimport transformed data.** `project/domain/plate_selection.rs::Project::plate_view`
  currently calls `selected_import_objects` for an explicit plate. Outside its `ImportScope::Plate`
  fast path, that helper restores `import_instances`, then `project/domain/shared_mesh.rs::materialize_meshes`
  and volume normalization can replace the edited geometry, transforms and instance placement.
  Retain the established `ImportScope::Plate(N)` and take the view from the live objects and
  instances. For a synthesized first plate with no saved importer filter, use the live all-admitted
  population for the explicit view, not a fresh importer pass. Do not fix this by switching the
  positive route to the instance-label filter used for all-plates views: it drops other instances of
  an admitted object before volume classification. Keep the original plate index for tower and
  plate option projection, and use `plater/slice_tower_fit.rs::fit_slice_wipe_tower` on the resulting
  model for each selected plate. This is the slice fit, separate from arrange-time tower placement.
- **Keep failure stages distinct.** `project_slice/state.rs::apply_plate` owns the first volume
  check and subsequent per-plate admission; `prepare_loaded_project_slice_with_validation` and
  `slice_cli_project_plates` reuse it. A forced current-plate arrange that needs another bed is
  `CLI_OBJECT_ARRANGE_FAILED` at plate `0` (`OrcaSlicer.cpp:6021–6034`). Attach the existing
  no-rotations text only to the initial volume failure on a slice plate when packing actually ran
  without rotations (`:6011–6013,6562–6573`); an arrange request alone is insufficient. Do not
  remap an importer-empty, post-skip, or unrelated apply failure. Carry the affected plate ID out of
  preflight, preserving the all-selected-plates check before publication. Extend core `SliceError`
  in `slice_error.rs` and `ares-cli` `failure/mapping.rs::slice_error_to_failure`; retain
  `failure/report.rs::CliFailure::with_plate_batch` and the existing publication order and reports.

`ProjectSource`, `CliProjectTransforms` and the existing prepare/apply calls provide the Rust seams
for these upstream stages. In particular, the old combined `CliProjectTransforms::apply(project,
plate)` is only a compatibility shell: do not carry over its calls to `prepare_import_scope`
inside the transform loop or arrange path. Inspection can use the model-action stages without
constructing a slice view. Neither loaders nor CLI output adapters need a second copy of model
geometry or a separate packer.

## Re-port order and reusable cases

Restore in the order below. Paths in the last column are relative to
`crates/ares-cli/tests/orca_parity/` for test modules and `tests/parity/` for fixtures. Reuse the
listed fixtures byte-for-byte (including runtime generators as written) and retain the existing
CLI case inputs, assertions and comparators. Compose the shared `case_support.rs` registrations,
helper visibility and request literals against the resulting tree; do not replace the case registry
or replay overlapping request/plumbing changes wholesale.

| Slice, in dependency order | Carry forward | Re-derive at the current seam | Reuse |
| --- | --- | --- | --- |
| Saved-project arrange, scale and Z rotation | Flag semantics, ordered Native scalar handling, volume math and recentering (`Model.cpp:1751–1757,1883–1894,1950–1964,3054–3101` under `libslic3r/`), CLI arrange defaults and failure variants. | All three source adapters, admitted-model actions on both CLI routes, signed ID precedence, stage-specific diagnostics and post-arrange plate enumeration. Remove duplicate orientation calls. | `cli_project_transforms.rs` cases (control, arrange `0`/`1`/auto, scale/Z and composition, invalid scale, selected/global failures); `orca_cli_ender3.3mf`. Keep its CLI/core stream comparison; adapt only request construction if necessary. |
| X/Y rotation and explicit grounding | X/Y volume math and recentering; `ensure_on_bed` runs after arrange, uses first-instance minimum model-part Z and moves all instances of the object, honoring object `auto_drop` over legacy build-item flags (`libslic3r/Model.cpp:1632–1672,1883–1927`; `Format/bbs_3mf.cpp:2263–2288,4131–4132,4486–4503` under `libslic3r/`). | Extend the ordered action stage and post-arrange stage separately; apply grounding to live instances before the final view, never to restored import instances. | `cli_project_transforms/tilt.rs`: both axes, repeated-key order, grounding after arrange, explicit false, standalone grounding and the two runtime auto-drop archive edits; original Ender-3 fixture. |
| Saved-project arrange modifiers | `--allow-rotations`, explicit Y alignment versus the i3 default, multicolor grouping, and the direct-project calibration-region no-op (`OrcaSlicer.cpp:1597–1626,5957–6013`; `libslic3r/Arrange.{hpp,cpp}` `ArrangeParams` and axis alignment/packing). | Resolve modifiers against loaded machine/plate settings at the shared arrange-job call. Return whether the job ran without rotations for preflight; modifiers alone do not force a job. | `cli_arrange_modifiers.rs`, including same-executable calibration toggle; `cli-arrange-modifiers/{rotation,alignment,multicolor}.3mf`. |
| Saved-project face grounding | `libslic3r/LayOnFace.cpp:21–242`: transformed model-part hull, eligible planes, largest/normal/point selection, normal and point validation, per-instance XY preservation and exact Z drop. Preserve no-plane/miss distinctions. | Run in ordered model actions over the admitted object population, before orientation and arrange. Point success is decided across admitted instances; later plate projection cannot undo lay-on-face. | `cli_ground_face.rs`, including short-edge coplanar multipart and rotation/point order; `orca_cli_ground_box.3mf`, `orca_cli_ground_multipart.3mf`. |
| Saved-project skip and assignment options | Signed list append semantics, clone/filament-assignment prerequisites and saved-3MF rejection (`OrcaSlicer.cpp:1650–1706,1724–1748`); loaded `identify_id` matching, unknown-ID no-op (`libslic3r/Format/bbs_3mf.cpp:2465–2498,4739–4744,4996–5004`). | Keep assignment validation before source ingestion in `main.rs::run_slice` using `project_slice/cli_object_selection.rs::CliObjectAssignment`. Move skip from the old single-output prepare flow into the `state.rs::apply_plate` phase, after `state/print_volume.rs::update_print_volume_state` and before remaining validation/config consumption. Distinguish initial outside/empty from `CLI_NO_SUITABLE_OBJECTS_AFTER_SKIP` and use the failing plate in multi-plate reports. | `cli_object_selection.rs`, including loaded ID versus resource ID, repeated lists, all skipped and assignment precedence; `orca_cli_object_selection.3mf`. |
| Paint and model inspection | Action admission/conflicts, `--info` model-part bounds/statistics, and `--inspect-paint` mesh-local annotation traversal/JSON (`OrcaSlicer.cpp:1427–1448,6335–6340,6364–6387`; `libslic3r/Model.cpp:1690–1703,2564–2603`; `slic3r/Utils/PaintCLI.cpp:26–204`). | Restore `ares-cli` `inspect.rs::run` and core `project/domain/inspection.rs::load` on the model-action stages without invoking `apply_plate` or changing slice or file-preset dispatch. Preserve archive materialization and paint payloads; do not implement transforms in the formatters. | `cli_model_inspection.rs`, `orca_cli_inspection_paint.3mf`, and its synthetic `open_box()` STL. |
| Ordered Qt hull and mesh inspection | Pure Rust 3D Qt specialization of `libslic3r/TriangleMesh.cpp:1358–1434` (dependency pin in `deps/Qhull/Qhull.cmake`), ordered packed vertices, triangle winding, two-stage part/object hull and shared `LayOnFace.cpp` planes. Keep `slic3r/Utils/MeshInspect.cpp:30–63` schema, including first-instance fields, plane order and limits. | Restore `geometry/quickhull.rs::convex_hull_3d` and fallible consumers in `project/domain/lay_on_face.rs` and `plater/auto_orient/mesh.rs`. Wire `--inspect-mesh` (`OrcaSlicer.cpp:1407–1425,6341–6363`) through the re-port of `inspection::load` so bounds and planes see the live placements. No new import scope in hull algorithms. | `cli_mesh_inspection.rs`, using the same paint 3MF (rotated multipart) and `open_box()` STL. Re-run existing public orientation geometry and face-grounding cases because the hull is shared. |
| Qt equal-vertex facets and neighbor linking | Preserve equal-vertex facets unless the Qt null-facet relink actually requests mirror handling; keep ordered indices, winding and inherited planes. Keep explicit errors for unsupported mirror deletion, forced duplicate-ridge merges and precision failures. With the dependency pin above, see Qhull `poly2_r.c:3482–3637,3745–3768` and `merge_r.c:471–484`. | Restore `geometry/quickhull/qt/{output,triangulation_links}.rs` on the ordered Qt port. There is no new importer behavior; re-run its inspection witness on the restored inspection load path, with the exact Native placement assertion. | `cli_narrow_hull_inspection.rs`, `qt-narrow-hull/narrow-fan.stl`, and `open_box()` control. Retain the case's specific Native narrow-hull stderr check and exact stdout and inventory comparisons. |
| STL placement before inspection | `OrcaSlicer.cpp:1749–1758,1924–1926,1969–1978,5051–5066,6335–6387`: `--arrange 0` preserves fresh STL XY for reports, not the independent Z drop. Preserve the enabled/default placement and report-specific coordinate frames. | Implement only in `project/domain/inspection.rs::load` after the shared action plumbing is restored; the existing automatic STL inspection centering is a compatibility shell, not a saved-project placement rule. No change to 3MF or ordinary STL slicing. | `cli_inspection_arrangement.rs`, reusing `open_box()` for the `--inspect-mesh`, `--info`, and `--inspect-paint` comparisons with both arrange settings. |

## Composition tests and limits

- Keep the existing `cli_slice_plates.rs` cases and fixtures, especially
  `custom-gcode-overview/two-plates.3mf`, `mesh-edge-pairing/odd-rotated.3mf`, and the later-plate
  preflight case. Add paired public CLI cases that combine transforms with `--slice 0` and positive
  selection across Embedded, Catalog and Files. Use synthetic inputs with a shared object and a separate object on a different plate;
  prove that valid positive selection retains the shared object's admitted instances and does not
  transform an excluded object's geometry. A face point that can only succeed on the excluded
  object gives an observable admission witness. Include loaded-profile bed shrink and distinct
  tower positions so forwarding `None` or projecting the wrong tower entry changes the result.
- Compare invalid scale plus invalid slice ID (distinguish the two invalid-parameter diagnostics,
  not just their status), and invalid slice ID plus a valid transform with forced arrange that
  would fail on a valid plate. Verify global arrange followed by all-plates output uses the final
  membership; include a later-plate initial volume failure after no-rotation packing and a
  later-plate all-skipped failure. Neither may publish an earlier plate's G-code. Check the reason,
  failing plate, and output inventory, not just exit status.
- Use live Native and replay in both official and seeded modes. Compare complete ordered G-code
  (existing generator-only normalization), Linux `result.json` (existing wall-clock normalization),
  diagnostics, output names/order and presence, and exact inspection bytes where specified.
  Do not commit Native output or loosen a comparator. Re-run shared hull/orientation consumers
  when the hull implementation changes and check workspace Clippy, formatting and wasm32.
- Deferred: generalizing automatic arrange for saved inputs beyond the existing supported decisions,
  duplication and positive clone/filament assignment, assembly-list layout and its calibration
  obstacle, exported-project thumbnail/skip annotation updates, general STL placement for slicing,
  unrelated export modes, GUI/WASM action entry points and Qhull modes outside the pinned 3D Qt
  path. Undefined upstream behavior is not a port target.
