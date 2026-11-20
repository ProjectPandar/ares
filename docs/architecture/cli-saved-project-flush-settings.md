# Prepare saved-project flush settings before CLI export

## OrcaSlicer boundary

In OrcaSlicer v2.5.0 (`40ecc66f7`), `src/OrcaSlicer.cpp:3849–4023`
prepares the project palette and rebuilds the flush matrix when required,
including when the matrix is absent. This precedes completion of the FFF
config defaults (`:4248–4259`) and `--export-settings` serialization
(`:6326–6333`). `PrintConfig.cpp:8267–8285` supplies the registered flush
vector and matrix defaults, not the finished matrix once preparation runs.

## Rust boundary

In `crates/ares-core/src/project/settings_export.rs::export_project_settings`,
reuse the CLI preparation in `options/flush_matrix.rs` before defaults are
filled and settings are serialized. Pass the resulting palette, matrix and
multipliers through to the existing writer. Preserve an explicitly supplied
valid matrix when no rebuild trigger fires. The existing prep and writer
remain the respective rewrite boundaries; do not add a second flush pipeline.

Do not change the shared JSON writer, purge formulas, registered defaults,
filament selection state, or nozzle mapping. Project archive export and
malformed selectors outside the defined flush domain are deferred.

## Verification

Compare live Native and Ares `--export-settings` output with identical
arguments on these saved FFF projects: one slot missing the matrix, two slots
missing the matrix, and a supplied valid matrix with no rebuild trigger.
Compare the matrix, vector, palette, multiplier and nozzle map as serialized.
