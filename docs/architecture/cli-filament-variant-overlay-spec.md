# CLI filament variant overlay boundary

## Status

Slice A specification. This slice ports the selected-profile CLI overlay boundary from OrcaSlicer 2.5.0. It is separate from the embedded BBL 3MF refresh performed by the direct project path; that work is deferred to Slice B.

## Upstream source boundary

The source boundary is `OrcaSlicer/src/OrcaSlicer.cpp`:

- `3510–3545`: validate or regenerate `filament_self_index`, derive old filament
  variant ranges, and normalize variant-aware vectors before refresh;
- `3639–3701`: derive variant indices and apply retained per-filament values
  with `set_with_restore_2`;
- `3753–3759`: apply newly supplied variant segments with the same restore
  operation;
- `3823–3845`: rebuild `filament_self_index` from the resulting per-filament
  variant counts;
- `Config.hpp:427–447`: vector assignment semantics used by the adjacent CLI
  overlay path.

The selected-profile identity and non-variant overlay remains in the adjacent
`OrcaSlicer.cpp:2750–2812`, `3155–3161`, `3259–3271`, and `3293–3301`
blocks, but those blocks are not the source of the variant-segment algorithm.

The Rust destination is `crates/ares-core/src/project/domain/profile_input/overlay.rs`, specifically `merged_profile_raw`, `variant_values`, and the filament identity reconstruction in that function.

## Included behavior

1. For each selected filament variant option, concatenate the selected filament preset's complete variant segment in filament-slot order, corresponding to the Native variant-index and restore operations in `3510–3545`, `3639–3701`, and `3753–3759`.
2. Preserve the embedded tail for unloaded filament slots.
3. Rebuild `filament_self_index` coherently with the selected variant counts, corresponding to Native's final reconstruction in `3823–3845`, while preserving the embedded tail when selected slots do not cover the full embedded vector.
4. Preserve the existing non-variant per-slot `set_at` behavior and identity-key handling.
5. Keep the existing compatibility and dynamic-group behavior at the same overlay boundary.

The authorized B59-01 expectation is Native CLI behavior: `filament_self_index = 1,2,3,4`. The scoped profile/overlay witnesses cover the selected-profile overlay and processed configuration paths.

## Explicitly deferred behavior

This slice does **not** implement the Native BBL refresh for embedded-only 3MF projects. In OrcaSlicer 2.5.0 that is a separate boundary in `OrcaSlicer.cpp:2786–2817`, where `is_bbl_3mf && new_printer_name.empty()` resolves embedded filament system presets and populates `load_filaments_config`, followed by the refresh block beginning around `3510`. Ares' direct `slice_project_plate` path currently bypasses `ProjectPresetRefresh`; that gap is Slice B.

The three Native exclude-hull cases are therefore not Slice A witnesses. On the base, they are already F→F because the direct path has no BBL filament refresh. They must not be used to judge this selected-profile overlay slice.

## Verification boundary

Slice A must pass:

- B59-01 with Native CLI `filament_self_index = 1,2,3,4`; the evidence path is `tests/support/filament_self_index.rs` and the parity fixture/runner that invokes the official Orca CLI, with the authorized assertion in `crates/ares-core/tests/profile_filament_self_index.rs`;
- the existing scoped profile/overlay tests;
- formatting, clippy, and the wasm32 check;
- scoped base-vs-tip parity with no P→F regressions.

No production resource changes or GUI `full_fff_config` behavior are part of this slice.
