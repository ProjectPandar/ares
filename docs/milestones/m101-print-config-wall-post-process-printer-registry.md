# M101: PrintConfig wall loop, post-process, and printer identity registry

## Goal
Port the adjacent wall-loop, alternate-extra-wall, post-processing script, process role-change G-code, printer identity, and print/printer settings-id option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4918-4986` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1158-1159,1394,1547-1548,1634`, `PrintConfig.cpp:4918-4986`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wall-loop generation behavior, alternate-extra-wall behavior, post-processing script execution, process role-change G-code insertion, printer preset identity behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `wall_loops`, `alternate_extra_wall`, `post_process`, `process_change_extrusion_role_gcode`, `printer_model`, `printer_notes`, `printer_variant`, `print_settings_id`, and `printer_settings_id` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC; split near-limit registry shards when needed to preserve this limit.
- Runtime behavior for wall-loop generation, alternate extra wall planning, post-processing script execution, role-change G-code insertion, printer identity/preset semantics, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following support/raft/resolution/retraction options from `PrintConfig.cpp:4988+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
