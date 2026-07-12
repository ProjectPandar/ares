# M22: PrintConfig option registry foundation

## Goal
Port the first `libslic3r::PrintConfig` option-definition boundary into `ares-core` by adding a source-cited registry for options Ares already parses.

## Exit checklist
- `ares-core` exposes `OptionValueKind`, `OptionDefinition`, `option_definitions()`, and `option_definition(key)`.
- The registry includes every option currently parsed by `SliceOptions`, including layer, hardware, infill, width, speed, skirt, brim, and bridge keys.
- Each registered option records its upstream value kind, metadata default, and concrete `OrcaSlicer/src/libslic3r/PrintConfig.cpp` source anchor.
- Registry metadata does not change existing parsing defaults, validation, unknown-key preservation, generated G-code bytes, CLI behavior, crates, or dependencies.
- `SliceOptions` exposes a read-only known-definition count while preserving unknown keys.
- G-code writer/planner parity remains a later milestone after option registry groundwork.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
