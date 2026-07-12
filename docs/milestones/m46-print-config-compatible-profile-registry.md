# M46: PrintConfig compatible profile option registry

## Goal
Port the FFF compatible profile no-CLI option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1695-1748` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the upstream compatible-printer/profile metadata in `PrintConfig.cpp:1695-1748`; no new Ares pipeline, crate, dependency, compatibility-expression evaluator, preset filtering behavior, object override behavior, filesystem, network, UI, slicing, extrusion, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `compatible_printers`, `upward_compatible_machine`, `compatible_printers_condition`, `compatible_prints`, `compatible_prints_condition`, `compatible_machine_expression_group`, `compatible_process_expression_group`, `different_settings_to_system`, and `print_compatible_printers` with exact kinds, empty defaults, and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/tooltip/mode/CLI metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Compatibility-expression evaluation, preset filtering, project-file persistence semantics, profile composition behavior changes, object override behavior, UI behavior, and downstream slicing/G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
