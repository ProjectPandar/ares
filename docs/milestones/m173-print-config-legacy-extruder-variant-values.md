# M173: PrintConfig legacy extruder variant values

## Goal
Port the nozzle/extruder variant string replacement and `extruder_type` legacy normalization branches from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7959-7970` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7959-7970` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization replaces legacy string fragments `Normal` -> `Standard` and `Big Traffic` -> `High Flow` for the covered nozzle/extruder variant keys.
- `SliceOptions` deserialization replaces legacy string fragment `DirectDrive` -> `Direct Drive` for `extruder_type`.
- Non-string values for covered keys remain preserved unchanged.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:7971+`, including power-loss recovery and shell-thickness migrations, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
