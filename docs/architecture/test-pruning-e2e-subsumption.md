# Test pruning: assertion-level E2E subsumption

## Decision

Delete tests only when an existing E2E asserts the same observable contract, or the test checks only a language-level tautology. Exercising a code path is not sufficient. Preserve error, numeric, platform, comparator-rejection and alternate-input coverage not asserted by E2E. The testing rules in `AGENTS.md` apply to future work.

## Evidence

- `crates/ares-cli/tests/stock_project_whole_file/tests.rs::project_matches_initialized_orca_242_ordered_bytes` invokes the CLI on the unchanged KSR project with seed 0 and compares the complete output to the committed initialized Orca artifact. The comparator normalizes generator identity/date only; no comparator or fixture changes are part of this cleanup. The 27 removed project-slicing checks assert subsets/properties of those same bytes. Default/alternate-plate wrapper tests remain.
- `crates/ares-cli/tests/slice_gcode.rs::slice_stl_writes_output_with_stl_format` supplies `is_infill_first: true` and asserts actual layer-1 infill-before-perimeter ordering. This subsumes the removed true-only parser test; default-false and invalid-input tests remain.
- `model_object_status_stores_each_regions_status_variant` assigned a public field and immediately compared it with the assigned value; no production operation intervened. Constructor/default/status transition tests remain. Removing the assignment-only test exposes two intentionally unwired enum variants; the existing M275 module-level `expect(dead_code)` now also applies to test builds instead of requiring a fake test to construct those variants.

## Removed tests

- `crates/ares-core/src/project_slice/gcode_emit/cooling/tests.rs::task22o174_layer_cooling_matches_orca_slowdown_feedrate`
- `crates/ares-core/src/project_slice/gcode_emit/motion/path/tests.rs::dynamic_outer_wall_travels_to_the_unprocessed_first_point`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests.rs::task22o133_contiguous_same_speed_paths_do_not_repeat_feedrate`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests.rs::task22o134_overhang_role_uses_bridge_kinematics`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests.rs::task22o135_overhang_overlap_bands_split_and_slow_wall_segments`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests.rs::task22o136_dynamic_segments_keep_raw_path_endpoint_identity`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests.rs::task22o137_dynamic_overhang_rounds_original_speed`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests.rs::task22o138_cooling_removes_redundant_feedrate_commands`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests/geometry.rs::first_layer_seam_and_island_order_match_project_slice`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests/geometry.rs::first_layer_inner_perimeter_uses_source_aligned_seam`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests/geometry.rs::first_layer_second_inner_perimeter_uses_fitted_aligned_seam`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests/geometry.rs::first_layer_linear_extrusions_use_project_geometry_lengths`
- `crates/ares-core/src/project_slice/gcode_emit/motion/tests/geometry.rs::bottom_surface_travel_crossing_external_slice_retracts`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_project_motion_is_finite_and_uses_configured_first_layer_rates`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::task22o143_ksr_seam_gap_clips_before_path_simplification`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_machine_start_uses_skirt_expanded_first_layer_hull`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_layer_change_appends_fan_speed_marker_after_custom_gcode`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_project_renders_timelapse_template_after_every_layer`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_project_retracts_wipes_and_lifts_before_timelapse`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_layer_metadata_uses_orca_float_precision`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_internal_bridges_keep_their_processor_role`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::ksr_project_renders_end_templates_and_closes_executable_block`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::lifted_next_layer_travel_carries_new_layer_hop_z`
- `crates/ares-core/src/project_slice/gcode_emit/tests.rs::task22o162_project_emits_filament_statistics`
- `crates/ares-core/src/project_slice/gcode_emit/tests/object_labels.rs::ksr_object_travel_acceleration_follows_object_comment`
- `crates/ares-core/src/project_slice/gcode_emit/tests/object_labels.rs::ksr_project_emits_3mf_object_labels_per_layer`
- `crates/ares-core/src/project_slice/tests/plates.rs::single_plate_project_loads_all_its_instances`
- `crates/ares-core/src/options/tests/core.rs::parses_infill_first_boolean`
- `crates/ares-core/src/print_apply/tests/model_object_status.rs::model_object_status_stores_each_regions_status_variant`

## Coverage limits

Four parallel audits inventoried the first-party Rust test declarations and reviewed selected assertions against CLI/browser E2E assertions. This was not an exhaustive semantic proof for every test: unreviewed or uncertain candidates remain. Vendored/upstream tests, runtime behavior, comparators and reference artifacts are unchanged. Live-Orca sweeps are environment-gated and browser checks run separately; a default nextest pass does not establish full printer/option matrix parity.

## Verification and recovery

Baseline at `c3f61645`: `cargo nextest run --workspace` passed 7,618 tests with 9 skipped. After deletion: 7,589 passed with the same 9 skips. `cargo fmt --check`, WASM target checks/release build/bindgen, and all three Playwright browser tests passed. On this Nix host Playwright needed an `LD_LIBRARY_PATH` containing its x86_64 shared-library dependencies; the initial missing-library launch failure was retried, not skipped.

Strict `cargo clippy --workspace --all-targets --all-features -- -D warnings` failed on existing lint debt in unchanged code under both Rust 1.96 and the CI-pinned 1.91 (including `excessive_nesting`). The operator explicitly approved ordinary clippy success as this cleanup's publication gate, with the strict failures recorded rather than unrelated production-code refactoring. `cargo clippy --workspace --all-targets --all-features` exited successfully (253 core-library warnings, 269 core-test warnings including duplicates, plus adapter/test warnings); fixing those is a separate follow-up.

The retained native artifact `target/test-pruning/ksr.gcode` matches the complete committed initialized Orca reference after normalizing only the canonical generator line. Reproduce it with:

```sh
mkdir -p target/test-pruning
cargo run -p ares-cli -- slice -o target/test-pruning/ksr.gcode tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf
sha256sum target/test-pruning/ksr.gcode
```

Expected SHA-256: `91e2fce752e5dc6d7ed221b39209838000ba1cd1bf134e937a59a8b8d2d27ab4`. Browser parity also retains its complete `browser.gcode` and prints its path. Source-only deletions are reversible from the commit diff; no data migration is involved.
