# Direct STL sliced 3MF export

## Upstream boundary

This slice ports the source-format-independent sliced project export owned by
OrcaSlicer v2.5.0:

- `OrcaSlicer/src/OrcaSlicer.cpp:7345` calls
  `PartPlateList::store_to_3mf_structure` after slicing.
- `OrcaSlicer/src/OrcaSlicer.cpp:8061-8081` calls `CLI::export_project`.
- `OrcaSlicer/src/OrcaSlicer.cpp:8369-8405` maps that operation to
  `Slic3r::store_bbs_3mf(StoreParams&)`.
- `OrcaSlicer/src/libslic3r/Format/bbs_3mf.cpp:6320-6350` derives export
  strategy flags and delegates to `_save_model_to_file`.
- `OrcaSlicer/src/libslic3r/Format/bbs_3mf.cpp:6420-6685` writes the model,
  plate metadata, project settings, and sliced G-code members.

`store_bbs_3mf` receives a placed `Model`; it is independent of whether the
model originated from STL or 3MF input.

## Rust destination

The destination is the existing shared package writer in
`crates/ares-core/src/project/export.rs`, called by
`crates/ares-cli/src/export_3mf.rs`. The existing saved-project exporter is a
temporary compatibility shell around the upstream `store_bbs_3mf` boundary:
it already captures `ProcessedGCode::archive` and emits the shared
`write_package` members, but currently reloads geometry only from a saved 3MF
and rejects direct STL export before slicing.

The direct STL archive facts use the existing
`ProcessedGCode::archive` mechanism. Archive capture is enabled only at the
single-STL, non-arranged load-settings call; the arranged multi-STL loop is
outside this slice.

## Included behavior

- Accept one STL input with loaded machine, process, and filament settings for
  `--slice 1 --export-3mf`.
- Slice the already loaded and placed model, capture its plate archive facts,
  and package the same placed model geometry and sliced members through the
  existing shared writer.
- Preserve the strict archive member and G-code comparisons in
  `export_3mf_sliced.rs`.
- Preserve the saved 3MF single-plate export path and its existing validation.

## Deferred behavior

- Multi-STL arranged export remains deferred.
- Multi-plate and multi-instance sliced export remains deferred.
- Additional saved-project model features, including painted meshes and
  unsupported archive members, remain deferred as they are for the existing
  exporter.

No independent Ares export pipeline is introduced. The existing geometry and
archive scaffolding is extended only to carry the already loaded model into
the upstream-shaped shared package boundary.
