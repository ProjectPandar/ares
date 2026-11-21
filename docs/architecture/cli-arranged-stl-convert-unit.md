# Convert fresh STL units before arrangement and plate selection

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) resets
the early selection to all for fresh inputs (`src/OrcaSlicer.cpp:2042–2045`).
Its transform loop applies `--convert-unit` to every imported model
(`4909,4961–4972`). Even when `--slice` appears earlier in argv, arrangement
runs on the resulting geometry (`6011–6012`) and the slice action rereads
the requested plate number (`6441–6445,6522–6527`). Selecting plate 2 does
not restrict conversion or arrangement to that plate's eventual objects.

The `src/libslic3r/Model.cpp:975–1041` boundary is `Model::looks_like_saved_in_meters`
and `Model::looks_like_imperial_units`, followed by
`Model::convert_from_meters(true)` or `Model::convert_from_imperial_units(true)`.
For a fresh single-volume STL the order is solid volume `< 0.008` then
`< 8.0`, otherwise unchanged. Conversion scales mesh and offsets and records
original units. Apply the rule independently per imported file, not to the
aggregate arranged volume.

## Rust destination

In `ares-cli/src/direct_stl.rs`, permit the flag for the existing arranged
STL CLI route with loaded presets. At import, reuse
`ares-core/src/project/domain/stl_units.rs::ImportedStl` inference and
conversion for each file before passing geometry into the existing arranged
project path (`project/domain/multipart.rs` and
`profiles/entry/load_settings.rs::slice_stl_models_with_load_settings`). The
raw `Model` handoff in that route is a temporary shell around the merged
upstream model; the converted route must preserve the imported volume,
including its transform and unit provenance. Preserve input ordering and
object/instance identities. Do not reread or convert STLs after arrangement,
and leave the current single-STL direct slice semantics unchanged.

## Verification

Run the public CLI with identical argv in both engines, using two synthetic
short closed boxes in meters, each too wide to share the same bed once
scaled. `--slice 2 --arrange 1 --convert-unit` with loaded presets must
succeed and emit `plate_2.gcode`, not `plate_1.gcode`. Without conversion
the same inputs must not emit plate 2. Slice the equivalent millimeter
inputs without the flag as a control for placement and the selected plate's
geometry. Assert exit, output inventory, indexed result report and selected
geometry from the exported G-code. Exercise both Native modes and preserve
the existing direct single-STL conversion and arranged slice cases.

## Deferred

No changes to arrangement, unit heuristics, cut/multipart object eligibility,
saved-project conversion, unsupported meshes, or export-only actions.
