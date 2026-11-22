# Arranged multi-STL sliced 3MF export

## Upstream boundary

This slice ports the arranged direct-STL project export owned by OrcaSlicer
v2.5.0:

- `OrcaSlicer/src/OrcaSlicer.cpp:5011-5065` maps the CLI `arrange` value to
  the `need_arrange` decision, and `:5619-6005` runs the global arrangement,
  creating the placed objects and plates.
- `OrcaSlicer/src/OrcaSlicer.cpp:7340-7371` builds the sliced plate data for
  export, `:8081-8086` dispatches `CLI::export_project`, and `:8377-8405`
  forwards the placed model, project settings, and every plate's data to
  `Slic3r::store_bbs_3mf(StoreParams&)`.
- `OrcaSlicer/src/slic3r/GUI/PartPlate.cpp:8422` owns
  `PartPlateList::store_to_3mf_structure`, which materializes the per-plate
  metadata after slicing.
- `OrcaSlicer/src/libslic3r/Format/bbs_3mf.cpp:6327-6350` derives the
  `WithGcode`/split-model export strategy and calls `_save_model_to_file`;
  `:6789-6826` writes each plate's MD5 and G-code members; and `:9005-9068`
  adds the valid `Metadata/plate_N.gcode` members. The public
  `store_bbs_3mf` boundary is at `:9723-9734`.

Native's two arranged STL inputs produce one placed model with a plate list.
For the witness geometry, `large.stl` is plate 1 and `medium.stl` is plate 2;
each plate has its own `Metadata/plate_N.json`, MD5, and G-code member.
Native's parallel staged G-code insertion can change the central-directory
order of the `plate_N.gcode` members. This is an observed Native packaging
property, not a reason to invent a second Ares writer.

## Rust destination

The destination remains the existing direct-STL and shared package boundaries:

- `crates/ares-cli/src/direct_stl.rs` continues to read all positional STL
  inputs, load the supplied settings, and dispatch the arranged work.
- `crates/ares-core/src/project_slice/cli_stl.rs` remains the sole arranged
  STL path. Its `slice_stl_models_with_load_settings` prechecks all selected
  plates, pulls each `AppliedPlate`, and returns `SlicedStlPlate` values in
  plate order. The export operation extends this path to retain the
  `ProcessedGCode` archive facts for every plate rather than slicing each
  input through a new export-only pipeline.
- `crates/ares-core/src/project_slice/archive_plate.rs` and
  `project_slice/gcode_emit/archive.rs` remain the archive-fact boundary. The
  direct-STL archive source supplies placed object geometry, identify IDs,
  bounds, layer heights, filament metadata, and effective project settings for
  each plate.
- `crates/ares-core/src/project/export.rs::write_package` remains the only
  ZIP/package writer. It must accept the arranged object list and a plate-data
  list, emit each plate's metadata and G-code/MD5 members, and preserve the
  existing single-plate and saved-project callers. CLI remains responsible for
  reading STL/settings inputs and writing the returned bytes to the requested
  output path.

The current saved-project and single-STL exporters are temporary compatibility
shells around the upstream `store_bbs_3mf` boundary. They are not alternate
multi-STL implementations. The existing `Object::from_placed_stl` geometry
conversion and XML helpers should be reused for the arranged placed model.

## Included behavior

- Accept multiple STL inputs with loaded machine, process, and filament
  settings for `--arrange 1 --slice 0 --export-3mf`.
- Use the arranged model and all prechecked plates produced by
  `slice_stl_models_with_load_settings`; do not re-read or independently
  center the STL inputs during export.
- Capture one sliced archive record per completed plate, including the
  plate-specific object facts, effective settings, plate metadata, G-code, and
  MD5. A later plate failure must preserve the existing direct-STL precheck and
  publication semantics rather than publishing a partial archive.
- Package all arranged STL geometry and every completed plate through the
  existing `write_package`: split model members, plate JSON records,
  `Metadata/plate_N.gcode.md5`, `Metadata/plate_N.gcode`, model settings,
  slice info, filament sequence, relationships, and project settings.
- Keep plate numbering and object identity in the upstream order. The witness
  must identify the large object on plate 1 and the medium object on plate 2,
  with nonempty G-code for both plates.
- Preserve existing one-STL sliced export, saved-project sliced export, and
  unsliced project export behavior.
- Add public-API E2E parity coverage using identical argv for Native and Ares in
  official and seeded environments. Compare member names and payloads, with a
  documented treatment of the proven Native G-code member-order nondeterminism;
  do not mask payload differences or sort away a difference without first
  proving it is solely Native archive ordering.

## Deferred behavior

- Multiple instances of one STL, modifiers, painted meshes, non-model volumes,
  custom archive members, thumbnails, and additional saved-project metadata.
- Mixed saved-3MF plus STL inputs, explicit plate selection combined with
  arranged multi-STL export, orientation, and transforms other than the
  existing `--arrange 1` path.
- New arrangement heuristics, a second direct-STL slicer, or a second ZIP
  writer. Arrangement remains the behavior already owned by
  `project_slice/cli_stl.rs` and its upstream `PartPlate`/arrangement boundary.
- Native's incidental ZIP envelope fields and its nondeterministic order among
  concurrently staged G-code members. The test may account for that order only
  after repeated same-input Native runs establish the exact nondeterminism; all
  member bytes, names, plate numbering, and non-G-code member order remain
  strict.

No independent Ares export pipeline is introduced. The existing arranged-STL
slice path is extended only to carry its already placed model and per-plate
archive facts into the upstream-shaped shared `write_package` boundary.
