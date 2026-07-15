#[cfg_attr(not(test), expect(dead_code, reason = "M351 staged before wiring"))]
mod apply_extruder_count_change_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M344 staged before wiring"))]
mod apply_print_diff_config_invalidation_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M343 staged before wiring"))]
mod apply_status_initial_diff_update_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M342 staged before wiring"))]
mod apply_print_diff_set_reassign_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M341 staged before wiring"))]
mod apply_manual_filament_map_same_map_prune_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M340 staged before wiring"))]
mod apply_manual_filament_map_setup_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M339 staged before wiring"))]
mod apply_auto_filament_map_diff_prune_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M338 staged before wiring"))]
mod apply_filament_map_auto_mode_gate_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M337 staged before wiring"))]
mod apply_filament_map_mode_guard_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M329 staged before wiring"))]
mod apply_scarf_joint_seam_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M328 staged before wiring"))]
mod apply_support_used_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M335 staged before wiring"))]
mod apply_filament_map_extraction_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M273 staged before wiring"))]
mod instance_sync_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M282 staged before wiring"))]
mod mesh_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M284 staged before wiring"))]
mod volume_cache_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M279 staged before wiring"))]
mod model_volume_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M280 staged before wiring"))]
mod transform_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M287 staged before wiring"))]
mod print_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M300 staged before wiring"))]
mod painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M306 staged before wiring"))]
mod fuzzy_painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M307 staged before wiring"))]
mod region_merge_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M288 staged before wiring"))]
mod verify_update_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M293 staged before wiring"))]
mod verify_update_config_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M275 staged before wiring"))]
mod model_object_status_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M277 stages PrintObjectStatus"))]
mod print_object_status_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M314 staged before wiring"))]
mod generate_regions_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M317 staged before wiring"))]
mod generate_model_part_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M318 staged before wiring"))]
mod generate_modifier_parent_scan_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M319 staged before wiring"))]
mod generate_modifier_changed_config_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M320 staged before wiring"))]
mod generate_modifier_unchanged_fallback_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M321 staged before wiring"))]
mod generate_painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M322 staged before wiring"))]
mod generate_painted_region_sort_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M323 staged before wiring"))]
mod generate_fuzzy_volume_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M324 staged before wiring"))]
mod generate_fuzzy_painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M345 staged before wiring"))]
mod apply_full_config_placeholder_entry_state;
#[cfg_attr(not(test), expect(dead_code, reason = "M325 staged before wiring"))]
mod generate_fuzzy_painted_region_sort_state;
