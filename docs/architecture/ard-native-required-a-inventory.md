# Native-required slice A — file boundary inventory

Availability contract: [ARD](ard-native-required-a.md). This inventory names test
source consumers, not Native output files. Reference byte/data assertions and
production code are outside the change boundary.

- Central policy: missing mandatory Native fails; optional absence reports SKIPPED.
- Gate: retain explicit selection, check mandatory availability before return,
  and label nonexecuted Native comparisons SKIPPED.
- Harness: synthetic launcher/cache/path subprocess contracts are not live Native
  evidence; per-child environments remain isolated and explicit.

| File | Migration boundary |
| --- | --- |
| `crates/ares-cli/tests/orca_parity/adaptive_pressure_advance.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/adaptive_pressure_advance/tools.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/bridge_over_infill_filament.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/brim_bed_region.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/change_filament_flush_blocks.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/curled_perimeter_slowdown.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/empty_selected_plate.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/empty_selected_plate_controls.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/filament_usage_stats.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/gcode_printable_area.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/honeycomb_multiline.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/klipper_velocity_limit.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/m205.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/narrowed_bridge_anchor_whole_file_control.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/air_filtration.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/bookkeeping_export.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/bottom_shell_count.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/chamber_control.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/cut_retraction.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/cut_retraction_values.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/elephant_foot_count.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/filament_ironing_speed.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/filament_map_mode.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/filament_printable.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/filament_stats_vectors.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/filament_variant_zero_fill.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/filename_format.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/fill_grid.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/hot_plate.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/initialized_bridge_direction.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/initialized_bridge_direction/rehash.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/initialized_bridge_direction/tilted.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/initialized_object_id.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/initialized_short_line.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/initialized_stitch_junction.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/initialized_transition_snap.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/ironing_fan.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/lightning_output.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/locked_widths.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/monotonic_chain.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/normal_support_output.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/normal_support_output/angle.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/normal_support_output/cooling.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/normal_support_output/dependencies.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/normal_support_output/interface_fan.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/normal_support_output/style.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/obsolete_silent_mode.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/owner_pair_tests.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/preset_identity_tests.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/pressure_advance.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/resolution_floor.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/scalar_points.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/seeded_context.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/single_volume_offset.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/skirt_height.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/slow_layer_count.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/support_report_tests.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/sweep.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/sweep_tests.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/thumbnails.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/top_shell_count.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/option_coverage/tree_activation.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/option_coverage/vitrification.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/bed_compatibility.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/bed_compatibility_native_controls.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/bridge_width.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/c1_thresholds.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/layer_nozzle.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/maximum_width.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/minimum_width.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/print_validation/positive.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/arachne_open_wall_order.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/cache_contract_tests.rs` | Harness audit; preserve assertions |
| `crates/ares-cli/tests/orca_parity/runner/clipped_loop_motion.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/end_layer.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/infill_first.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/inherited_filament.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/modifier_travel.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/same_xy.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/auto_lift.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/change.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/cooling.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/detour.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/filament_stats.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/fixture.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/seam.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/timing.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/embedded_script_whole_file.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/empty_tools.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/filament_map_ramming.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/filament_self_index.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/generated_printer_ids.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/infill_first_ironing.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/initialized_loop_chain.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/initialized_overhang_intersections.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/j1_used_extruder_mask_follows_emitted_tools.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/avoid_crossing_routes_around_top_surfaces.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/loaded_config_acceptance.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/loaded_config_probes.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/loaded_config_ranges.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/loaded_config_validation.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/multi_object_skirt.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/nearest_seam_gaussian.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/object_layer_height_ranges.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/overhang_reverse.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/path_tests.rs` | Harness audit; preserve assertions |
| `crates/ares-cli/tests/orca_parity/runner/preset_scalar_array_slice.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/preset_scalar_loading.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/preset_vector_loading.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/profile_entry.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/profile_entry_3mf.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/profile_inherits_group.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/profile_legacy_overlay.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/profile_overlay_default_gcodes.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/project_export_metadata.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/scarf_default_threshold.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/seam_layer_height.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/seam_object_bias.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/small_loop_speed.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/stl_model_label.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/stl_variant_whole_file.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/support_annotations.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/support_modifier.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/support_modifier/bridge_flow.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/template_rng.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/tree_support.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/tree_support/manual.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/tree_support/manual/blocker.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/tree_support/manual/split.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/tree_support/manual/split_sides.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/tree_support/manual/styles.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/tree_support/manual/vertical.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/runner/tree_support/retraction.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/user_overrides.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/runner/flush_matrix_cli_reconstruction.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/sermoon_v1_whole_file.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/smoke.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/smoke/nozzle.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/smoke/sweep.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/smooth_type2_tower.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/smooth_type2_tower/chamber.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/smooth_type2_tower/conflict.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/smooth_type2_tower/islands.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/smooth_type2_tower/tools.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/smooth_type2_tower/variants.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/stock_presets/sweep.rs` | Gate + central policy |
| `crates/ares-cli/tests/orca_parity/stock_presets/tests.rs` | Harness audit; preserve assertions |
| `crates/ares-cli/tests/orca_parity/stock_presets/timelapse.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/sync_time.rs` | Central policy; no caller rewrite needed |
| `crates/ares-cli/tests/orca_parity/vg1_flush_state.rs` | Central policy; no caller rewrite needed |

The separately enumerated procgcode/core mixed-test consumers and configuration
files are in the ARD table. No test module under this inventory is production
slicing code. Additional nonstandard gate forms must be classified against the
same contract; no missing Native environment can be classified as parity PASS.
