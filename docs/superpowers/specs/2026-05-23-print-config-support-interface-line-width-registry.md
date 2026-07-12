# M119 Spec: PrintConfig support interface base avoidance and line width registry slice

## Goal
Port the adjacent support interface base-avoidance and support line-width option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata, while mechanically splitting the near-limit support registry shard.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:961`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6036-6041`: `support_interface_not_for_body` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:960`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6043-6053`: `support_line_width` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/maximums/modes and ratio metadata beyond the current registry metadata boundary.
- Support interface filament routing, support/raft base material selection behavior, support line-width resolution, nozzle-diameter ratio computation, and support geometry.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6055+`: `support_interface_loop_pattern` and following support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: add new sorted shard modules between the existing tail-final and terminal suffix merge positions.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal.rs`: retain only the pre-support terminal prefix (`staggered_inner_seams` through `supertack_plate_temp_initial_layer`).
- Create `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: move the existing `support_*` definitions there, add `support_interface_not_for_body` and `support_line_width` in sorted order, and preserve all existing support metadata unchanged.
- Create `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: move the post-support terminal suffix (`symmetric_infill_y_axis` through `zaa_minimize_perimeter_height`) there unchanged.
- Registry key, metadata, fixture-count, and public lookup tests cover both new definitions.
- `docs/roadmap.md` and `docs/milestones/m119-print-config-support-interface-line-width-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_interface_not_for_body` (`coBool`, default `true`, field at `PrintConfig.hpp:961`, definition lines 6036-6041, Ares kind `Bool`)
- `support_line_width` (`coFloatOrPercent`, default `0`, field at `PrintConfig.hpp:960`, definition lines 6043-6053, Ares kind `FloatOrPercent`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support interface filament routing behavior, support line-width behavior, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_interface_loop_pattern` or following options from `PrintConfig.cpp:6055+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible after the mechanical shard split.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6055+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
