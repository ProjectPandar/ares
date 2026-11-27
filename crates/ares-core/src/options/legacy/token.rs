//! `PrintConfigDef::handle_legacy` (`PrintConfig.cpp:8033-8290`) and
//! `handle_legacy_composite` (`PrintConfig.cpp:8290-…`) ported to the
//! string-token semantics upstream uses: `set_deserialize`
//! (`Config.cpp:577-579`) hands each `key/value` pair's serialized token to
//! `handle_legacy(opt_key, value)`, which may rename `opt_key`, rewrite
//! `value`, or clear `opt_key` to drop the entry.
//!
//! One implementation shared by the `serde_json::Value` view (`SliceOptions`
//! deserialize) and the `RawConfigValue` view (`project_settings_raw`): each
//! adapter supplies the value's serialized token and stores the normalized
//! `key`/`token` back. `opt_key`/`value` cleared means drop the entry.

use std::collections::BTreeSet;

/// `PrintConfig.cpp:8033-8290`: per-entry legacy normalization on the
/// serialized `key`/`value` tokens. `key`/`value` are rewritten in place;
/// clearing `key` (obsolete or unrecognized) drops the entry.
///
/// Mirrors the upstream if-else chain; unknown keys are left to the
/// `print_config_def.has` terminal check the caller performs.
/// The now-absolute keys whose old-profile percentage values are erased
/// at load (`PrintConfig.cpp:8087-8094`).
pub(crate) fn erases_percentage_for_absolute_keys(opt_key: &str) -> bool {
    matches!(
        opt_key,
        "initial_layer_print_height"
            | "initial_layer_speed"
            | "internal_solid_infill_speed"
            | "top_surface_speed"
            | "support_interface_speed"
            | "outer_wall_speed"
            | "support_object_xy_distance"
    )
}

pub(crate) fn handle_legacy(opt_key: &mut String, value: &mut String) {
    match opt_key.as_str() {
        "curr_bed_type" if value == "SuperTack Plate" => *value = "Supertack Plate".to_owned(),
        "enable_wipe_tower" => *opt_key = "enable_prime_tower".to_owned(),
        "wipe_tower_width" => *opt_key = "prime_tower_width".to_owned(),
        "wiping_volume" => *opt_key = "prime_volume".to_owned(),
        "wipe_tower_brim_width" => *opt_key = "prime_tower_brim_width".to_owned(),
        "tool_change_gcode" => *opt_key = "change_filament_gcode".to_owned(),
        "bridge_fan_speed" => *opt_key = "overhang_fan_speed".to_owned(),
        "tpms_adaptive" => *opt_key = "adaptive_infill_mode".to_owned(),
        "tpms_interior_density" => *opt_key = "adaptive_infill_interior_density".to_owned(),
        "tpms_adaptive_gradient" => *opt_key = "adaptive_infill_gradient".to_owned(),
        "infill_extruder" => *opt_key = "sparse_infill_filament".to_owned(),
        "solid_infill_extruder" => *opt_key = "solid_infill_filament".to_owned(),
        "perimeter_extruder" => *opt_key = "wall_filament".to_owned(),
        "wipe_tower_extruder" => *opt_key = "wipe_tower_filament".to_owned(),
        "support_material_extruder" => *opt_key = "support_filament".to_owned(),
        "support_material_interface_extruder" => {
            *opt_key = "support_interface_filament".to_owned();
        }
        "support_material_angle" => *opt_key = "support_angle".to_owned(),
        "support_material_enforce_layers" => *opt_key = "enforce_support_layers".to_owned(),
        key if erases_percentage_for_absolute_keys(key) && value.contains('%') => {
            // Old profiles expressed these as percentages; the current keys are
            // absolute, so the entry is dropped to avoid a parse error.
            opt_key.clear();
        }
        "inherits_cummulative" => *opt_key = "inherits_group".to_owned(),
        "compatible_printers_condition_cummulative" => {
            *opt_key = "compatible_machine_expression_group".to_owned();
        }
        "compatible_prints_condition_cummulative" => {
            *opt_key = "compatible_process_expression_group".to_owned();
        }
        "cooling" => *opt_key = "slow_down_for_layer_cooling".to_owned(),
        "timelapse_no_toolhead" => *opt_key = "timelapse_type".to_owned(),
        "timelapse_type" if value == "2" => {
            // old file "0" is None, "2" is Traditional; new file "0" is
            // Traditional, erase "2".
            *value = "0".to_owned();
        }
        "support_type" => match value.as_str() {
            "normal" => *value = "normal(manual)".to_owned(),
            "tree" => *value = "tree(manual)".to_owned(),
            "hybrid(auto)" => *value = "tree(auto)".to_owned(),
            _ => {}
        },
        "support_base_pattern" if value == "none" => *value = "hollow".to_owned(),
        "tree_support_wall_count" if value == "-1" => {
            // PrintConfig.cpp (345efec64c7b):9839-9840: legacy auto, not a new minimum.
            *value = "0".to_owned();
        }
        "different_settings_to_system" => normalize_different_settings_to_system(value),
        "overhang_fan_threshold" if value == "5%" => *value = "10%".to_owned(),
        "wall_infill_order" => {
            // `PrintConfig.cpp:8134-8145`: the three-part legacy order collapses
            // to `wall_sequence`; the `infill/…` prefix forms additionally set
            // `is_infill_first` in `load_from_json` (Config.cpp:943-945), handled
            // at the composite level, not here.
            *opt_key = "wall_sequence".to_owned();
            *value = match value.as_str() {
                "inner wall/outer wall/infill" | "infill/inner wall/outer wall" => {
                    "inner wall/outer wall".to_owned()
                }
                "outer wall/inner wall/infill" | "infill/outer wall/inner wall" => {
                    "outer wall/inner wall".to_owned()
                }
                "inner-outer-inner wall/infill" => "inner-outer-inner wall".to_owned(),
                _ => return,
            };
        }
        "nozzle_volume_type"
        | "default_nozzle_volume_type"
        | "printer_extruder_variant"
        | "print_extruder_variant"
        | "filament_extruder_variant"
        | "extruder_variant_list" => {
            if value.contains("Normal") || value.contains("Big Traffic") {
                *value = value
                    .replace("Normal", "Standard")
                    .replace("Big Traffic", "High Flow");
            }
        }
        "extruder_type" => {
            if value.contains("DirectDrive") {
                *value = value.replace("DirectDrive", "Direct Drive");
            }
        }
        "enable_power_loss_recovery" => {
            if value == "1" || value.eq_ignore_ascii_case("true") {
                *value = "enable".to_owned();
            } else if value == "0" || value.eq_ignore_ascii_case("false") {
                *value = "disable".to_owned();
            }
        }
        "enable_arc_fitting" => {
            // PrintConfig.cpp (345efec64c7b):9886-9891.
            if value == "1" || value.eq_ignore_ascii_case("true") {
                *value = "enabled".to_owned();
            } else if value == "0" || value.eq_ignore_ascii_case("false") {
                *value = "disabled".to_owned();
            }
        }
        "ensure_vertical_shell_thickness" => {
            if value == "1" {
                *value = "ensure_all".to_owned();
            } else if value == "0" {
                *value = "ensure_moderate".to_owned();
            }
        }
        "rotate_solid_infill_direction" => {
            *opt_key = "solid_infill_rotate_template".to_owned();
            if value == "1" {
                *value = "0,90".to_owned();
            } else if value == "0" {
                *value = "0".to_owned();
            }
        }
        "sparse_infill_anchor" => *opt_key = "infill_anchor".to_owned(),
        "sparse_infill_anchor_max" => *opt_key = "infill_anchor_max".to_owned(),
        "chamber_temperatures" => *opt_key = "chamber_temperature".to_owned(),
        "thumbnail_size" => *opt_key = "thumbnails".to_owned(),
        "top_one_wall_type" if !value.is_empty() && value != "none" => {
            // `PrintConfig.cpp:8161-8165` uses `value != "none"`; the empty-value
            // guard preserves Ares's handling of non-string inputs (arrays, bools,
            // numbers are not renamed — only real string values trigger).
            *opt_key = "only_one_wall_top".to_owned();
            *value = "1".to_owned();
        }
        "initial_layer_flow_ratio" => *opt_key = "bottom_solid_infill_flow_ratio".to_owned(),
        "ironing_direction" => *opt_key = "ironing_angle".to_owned(),
        "ironing_angle" if value.starts_with('-') => *value = "0".to_owned(),
        "counterbole_hole_bridging" => *opt_key = "counterbore_hole_bridging".to_owned(),
        "draft_shield" if value == "limited" => *value = "disabled".to_owned(),
        "sparse_infill_pattern"
        | "top_surface_pattern"
        | "bottom_surface_pattern"
        | "internal_solid_infill_pattern"
        | "ironing_pattern"
        | "support_ironing_pattern"
            if value == "zig-zag" =>
        {
            *value = "rectilinear".to_owned();
        }
        "filament_map_mode" if value == "Auto" => *value = "Auto For Flush".to_owned(),
        "filament_type" => normalize_filament_type(value),
        "prime_tower_rib_wall" => {
            if value == "1" {
                *opt_key = "wipe_tower_wall_type".to_owned();
                *value = "rib".to_owned();
            } else {
                opt_key.clear();
            }
        }
        "prime_tower_extra_rib_length" => *opt_key = "wipe_tower_extra_rib_length".to_owned(),
        "prime_tower_rib_width" => *opt_key = "wipe_tower_rib_width".to_owned(),
        "prime_tower_fillet_wall" => *opt_key = "wipe_tower_fillet_wall".to_owned(),
        "extruder_clearance_max_radius" => *opt_key = "extruder_clearance_radius".to_owned(),
        "machine_switch_extruder_time" => *opt_key = "machine_tool_change_time".to_owned(),
        "wall_direction" if value == "auto" => *value = "ccw".to_owned(),
        key if is_obsolete_legacy_key(key) => opt_key.clear(),
        _ => {}
    }
}

/// `PrintConfig.cpp:8262-8288` obsolete-key set: these entries are dropped.
fn is_obsolete_legacy_key(key: &str) -> bool {
    matches!(
        key,
        "acceleration"
            | "scale"
            | "rotate"
            | "duplicate"
            | "duplicate_grid"
            | "bed_size"
            | "print_center"
            | "g0"
            | "wipe_tower_per_color_wipe"
            | "support_sharp_tails"
            | "support_remove_small_overhangs"
            | "support_with_sheath"
            | "tree_support_collision_resolution"
            | "tree_support_with_infill"
            | "max_volumetric_speed"
            | "max_print_speed"
            | "support_closing_radius"
            | "remove_freq_sweep"
            | "remove_bed_leveling"
            | "remove_extrusion_calibration"
            | "support_transition_line_width"
            | "support_transition_speed"
            | "bed_temperature"
            | "bed_temperature_initial_layer"
            | "can_switch_nozzle_type"
            | "can_add_auxiliary_fan"
            | "extra_flush_volume"
            | "spaghetti_detector"
            | "adaptive_layer_height"
            | "z_hop_type"
            | "z_lift_type"
            | "bed_temperature_difference"
            | "long_retraction_when_cut"
            | "retraction_distance_when_cut"
            | "internal_bridge_support_thickness"
            | "top_area_threshold"
            | "reduce_wall_solid_infill"
            | "filament_load_time"
            | "filament_unload_time"
            | "smooth_coefficient"
            | "overhang_totally_speed"
            | "overhang_speed_classic"
    )
}

/// `PrintConfig.cpp:8123-8133` `different_settings_to_system`: each `;`-
/// separated sub-key is itself run through `handle_legacy`; renamed keys are
/// replaced inside the value string.
fn normalize_different_settings_to_system(value: &mut String) {
    if value.is_empty() {
        return;
    }
    let unquoted = value.replace('"', "");
    let split_keys = unquoted
        .split(';')
        .filter(|key| !key.is_empty())
        .collect::<BTreeSet<_>>();
    let mut normalized = value.clone();
    for split_key in split_keys {
        let mut copy_key = split_key.to_owned();
        let mut copy_value = String::new();
        handle_legacy(&mut copy_key, &mut copy_value);
        if copy_key != split_key {
            normalized = normalized.replace(split_key, &copy_key);
        }
    }
    *value = normalized;
}

/// `PrintConfig.cpp:8186-8208` `filament_type`: `;`-separated tokens are
/// unquoted, `ASA-Aero`→`ASA-AERO`, and the value is rebuilt quoted when any
/// token changed.
fn normalize_filament_type(value: &mut String) {
    if !value.contains("ASA-Aero") {
        return;
    }
    let mut rebuild = false;
    let types = value
        .split_terminator(';')
        .map(|token| {
            let token = strip_surrounding_quotes(token);
            if token == "ASA-Aero" {
                rebuild = true;
                "ASA-AERO".to_owned()
            } else {
                token.to_owned()
            }
        })
        .collect::<Vec<_>>();
    if rebuild {
        *value = types
            .into_iter()
            .map(|token| format!("\"{token}\""))
            .collect::<Vec<_>>()
            .join(";");
    }
}

fn strip_surrounding_quotes(token: &str) -> &str {
    if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
        &token[1..token.len() - 1]
    } else {
        token
    }
}

/// Extra keys `load_from_json` derives alongside `handle_legacy`
/// (`Config.cpp:937-945`): a `wall_infill_order` value carrying an `infill/…`
/// order also sets `is_infill_first`. The derivation reads the source
/// `key`/`value` tokens — `handle_legacy` itself only rewrites the pair.
///
/// Returns the derived boolean assignment if the source pair produces one.
pub(crate) fn handle_legacy_derived(opt_key: &str, value: &str) -> Option<(&'static str, bool)> {
    if opt_key == "wall_infill_order"
        && matches!(
            value,
            "infill/inner wall/outer wall" | "infill/outer wall/inner wall"
        )
    {
        Some(("is_infill_first", true))
    } else {
        None
    }
}

/// Whether the post-`handle_legacy` destination key is a bool-valued option.
/// `SliceOptions`/`project_settings_raw` are untyped token maps, so a rewritten
/// `"1"`/`"0"` token is re-materialized as `Value::Bool` only when the
/// destination option is bool — for the SliceOptions keyspace that is the
/// `top_one_wall_type` → `only_one_wall_top` rewrite.
pub(crate) fn handle_legacy_dest_bool(opt_key: &str) -> bool {
    opt_key == "only_one_wall_top"
}
