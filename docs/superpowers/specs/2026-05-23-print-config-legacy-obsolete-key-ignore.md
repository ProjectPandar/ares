# M180 Spec: PrintConfig legacy obsolete-key ignore list

## Goal
Port the obsolete-key ignore set from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8069-8091`: obsolete configuration key ignore set and the branch that clears `opt_key` for ignored keys.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8093-8096`: final `print_config_def.has(opt_key)` unknown-key validation.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8099+`: `handle_legacy_composite` behavior and later code.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key dropping for the source-cited obsolete list.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M180 obsolete-key ignore check.
- `crates/ares-core/src/options/tests/legacy_obsolete_key_ignore.rs`: add focused M180 tests proving every obsolete key is dropped, dropping is value-type independent, unknown non-obsolete keys remain preserved, and prior M169-M179 migrations still precede ignore handling.
- `crates/ares-core/src/options/tests.rs`: register the M180 test module.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update the aggregate known-count expectation for obsolete keys that are intentionally dropped during deserialization.
- `docs/roadmap.md` and `docs/milestones/m180-print-config-legacy-obsolete-key-ignore.md`: milestone sequencing docs.

## Included obsolete keys

The following keys from `PrintConfig.cpp:8070-8086` are ignored:

- `acceleration`
- `scale`
- `rotate`
- `duplicate`
- `duplicate_grid`
- `bed_size`
- `print_center`
- `g0`
- `wipe_tower_per_color_wipe`
- `support_sharp_tails`
- `support_remove_small_overhangs`
- `support_with_sheath`
- `tree_support_collision_resolution`
- `tree_support_with_infill`
- `max_volumetric_speed`
- `max_print_speed`
- `support_closing_radius`
- `remove_freq_sweep`
- `remove_bed_leveling`
- `remove_extrusion_calibration`
- `support_transition_line_width`
- `support_transition_speed`
- `bed_temperature`
- `bed_temperature_initial_layer`
- `can_switch_nozzle_type`
- `can_add_auxiliary_fan`
- `extra_flush_volume`
- `spaghetti_detector`
- `adaptive_layer_height`
- `z_hop_type`
- `z_lift_type`
- `bed_temperature_difference`
- `long_retraction_when_cut`
- `retraction_distance_when_cut`
- `internal_bridge_support_thickness`
- `top_area_threshold`
- `reduce_wall_solid_infill`
- `filament_load_time`
- `filament_unload_time`
- `smooth_coefficient`
- `overhang_totally_speed`
- `silent_mode`
- `overhang_speed_classic`
- `filament_prime_volume`

## Functional requirements

1. Apply the obsolete-key ignore check when `SliceOptions` is deserialized from JSON.
2. Drop every included obsolete key regardless of whether its JSON value is string, number, bool, array, object, or null.
3. Preserve non-obsolete unknown options exactly as today because final registry-backed unknown-key validation is deferred.
4. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered obsolete legacy inputs are absent from stored values.
5. Keep the ordered M169-M179 legacy migrations intact; M180 must not change any prior key/value rewrite semantics.
6. Update aggregate known-count tests to reflect currently inserted obsolete known keys being dropped rather than counted as stored known values.
7. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
8. Do not implement final unknown-key validation from `PrintConfig.cpp:8093-8096` in this milestone.

## Acceptance checks

- Tests prove all obsolete keys from `PrintConfig.cpp:8070-8086` are removed from `SliceOptions::values()`.
- Tests prove obsolete keys are removed for representative non-string JSON values as well as strings.
- Tests prove non-obsolete unknown keys remain preserved.
- Tests prove at least one prior legacy alias and one prior value migration still work alongside obsolete-key dropping.
- Aggregate known-count tests reflect that `silent_mode` and `tree_support_with_infill` are obsolete keys and are dropped when inserted.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8093+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
