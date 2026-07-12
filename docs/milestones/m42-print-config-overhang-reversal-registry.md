# M42: PrintConfig overhang reversal option registry

## Goal
Port the FFF `overhang_reverse`, `overhang_reverse_internal_only`, `counterbore_hole_bridging`, and `overhang_reverse_threshold` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1446-1498` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its historical registry source boundary is `PrintConfig.hpp:401-403,1205-1208`, `PrintConfig.cpp:551-556,1446-1498`; no new Ares pipeline, crate, counterbore bridge generation, wall planning, extrusion behavior, filesystem, network, UI, preset behavior, or object override behavior was added by the original registry milestone.

## Runtime consumption update
As of the overhang-reverse consumption slices, Ares parses `overhang_reverse` and `overhang_reverse_internal_only` into `PerimeterOptions`. Ares reverses already-classified rectangular `PerimeterRole::Overhang` paths on zero-based odd layer ids, matching Orca's `PerimeterGenerator.cpp:108-109` and `374-375` `layer_id % 2 == 1` gate for even GUI layers. For rectangular multi-wall overhang contours, `overhang_reverse_internal_only` preserves the external overhang path direction and reverses generated internal perimeter paths, reflecting Orca's `PerimeterGenerator.cpp:1117-1141` external-path skip in the current Ares rectangular scaffold. These runtime slices reach downstream print paths, toolpath moves, and G-code through the existing Ares perimeter pipeline.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `overhang_reverse`, `overhang_reverse_internal_only`, `counterbore_hole_bridging`, and `overhang_reverse_threshold` with exact defaults and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream enum labels, label/category/tooltip/sidetext/min/max/ratio-over/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Full Orca `reorient_perimeters` parity remains deferred beyond the rectangular overhang path-order and internal-only multi-wall slices already consumed.
- `overhang_reverse_threshold`, `counterbore_hole_bridging`, hole-specific loop-role behavior, Arachne extrusion reversal, fuzzy-skin special reversal, thin walls, supports, raft-layer gates, partial overhang clipping, counterbore bridge/perimeter behavior, and full downstream print-planning behavior remain deferred.
- Following speed options and later quality/wall options remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
