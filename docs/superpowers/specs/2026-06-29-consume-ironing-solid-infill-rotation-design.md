# Consume Ordinary Ironing Solid Infill Rotation

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1096-1097` defines `solid_infill_direction` and `solid_infill_rotate_template` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1145-1146` defines `ironing_angle` and `ironing_angle_fixed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2868-2880` defines the `solid_infill_direction` default and range.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3887-3899` defines the solid infill rotation template option.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4231-4246` defines `ironing_angle` and `ironing_angle_fixed`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:52-80` defines `calculate_infill_rotation_angle(...)` and the empty-template fixed-angle path.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1598-1599` sets ordinary ironing angle to `(ironing_angle_fixed ? 0 : calculate_infill_rotation_angle(... solid_infill_direction, solid_infill_rotate_template)) + ironing_angle`, and marks the fill angle fixed when `ironing_angle_fixed` is true or the solid rotate template is non-empty.

## Ares Destination

- `crates/ares-core/src/options/ironing_type.rs` owns ordinary ironing option parsing and angle selection.
- `crates/ares-core/src/print_paths/ironing.rs` owns current ordinary ironing path duplication and rectilinear scanline generation.
- `crates/ares-core/src/pipeline/tests/ironing_angle.rs` and adjacent ordinary-ironing tests own behavior coverage.

## Required Behavior

- Ordinary rectilinear ironing must derive its scanline angle from Ares' parsed `solid_infill_direction` when `ironing_angle_fixed` is false and `solid_infill_rotate_template` is empty.
- Non-fixed ordinary rectilinear ironing must keep Ares' existing even/odd layer alternation, but the alternation starts from `solid_infill_direction` instead of a hard-coded zero-degree base.
- When `solid_infill_rotate_template` is non-empty, ordinary rectilinear ironing must use `template[layer_index % template.len()] + ironing_angle` and suppress the additional odd-layer 90-degree alternation, matching Orca's `fixed_angle = ... || !template.empty()` handoff.
- When `ironing_angle_fixed` is true, ordinary rectilinear ironing must keep absolute `ironing_angle` behavior and ignore `solid_infill_direction` plus `solid_infill_rotate_template`.
- Existing `ironing_angle` remains an offset added after the selected solid-infill base angle.
- Ordinary concentric ironing, support ironing, ironing inset/spacing/flow/speed, and non-rectangular duplicate behavior remain unchanged except where tests pin the previous hard-coded zero-degree base.

## Deferred Behavior

- Full Orca advanced rotate-template metalanguage from `Fill/Fill.cpp:52-80` remains deferred; Ares keeps its current simple comma-separated template parser.
- Full Orca `Fill` pattern internals, `ExtrusionEntityCollection`, ironing extrusion grouping, per-object layer-id subtleties, z-aware alternatives, and binary E2E parity remain deferred.
- No new options, crates, dependencies, UI behavior, support-ironing behavior, or Ares-owned pipeline design are added.

## Acceptance

- A test proves default ordinary top ironing with omitted `ironing_angle` now follows default `solid_infill_direction = 45` instead of Ares' previous horizontal baseline.
- A test proves `solid_infill_direction = 90` with `ironing_angle = 0` generates vertical ordinary ironing lines.
- A test proves `solid_infill_rotate_template = "90,0"` controls layer 0 and layer 1 ordinary ironing directions without odd-layer alternation.
- A test proves `ironing_angle_fixed = true` keeps absolute `ironing_angle` and ignores conflicting solid infill rotation settings.
- Existing spacing/inset/pattern tests that are not about angle explicitly set `solid_infill_direction = 0` where they depend on horizontal lines.
- Verification must run targeted nextest for affected tests plus the project standard checks before commit.
