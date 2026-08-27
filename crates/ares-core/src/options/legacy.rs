use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Deserializer, de::Error};
use serde_json::Value;

use super::SliceOptions;

mod sla;
mod thumbnails;
mod wiping_volumes;

impl<'de> Deserialize<'de> for SliceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = BTreeMap::<String, Value>::deserialize(deserializer)?;
        let values = normalize_legacy_options(values);
        let values = normalize_legacy_composite_options(values).map_err(D::Error::custom)?;
        Ok(Self { values })
    }
}

fn normalize_legacy_options(values: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut normalized = BTreeMap::new();
    for (key, value) in values {
        normalize_legacy_option_into(&mut normalized, key, value);
    }
    normalized
}

fn normalize_legacy_composite_options(
    mut values: BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>, String> {
    thumbnails::normalize_legacy_thumbnails(&mut values)?;
    wiping_volumes::normalize_legacy_wiping_volumes(&mut values)?;
    sla::normalize_legacy_sla(&mut values)?;
    Ok(values)
}

fn normalize_legacy_option_into(values: &mut BTreeMap<String, Value>, key: String, value: Value) {
    if key == "wall_infill_order" {
        if wall_infill_order_is_infill_first(&value) {
            values
                .entry("is_infill_first".to_owned())
                .or_insert(Value::Bool(true));
        }
        values.insert(
            "wall_sequence".to_owned(),
            normalize_wall_infill_order(value),
        );
        return;
    }

    if let Some((key, value)) = normalize_legacy_option((key, value)) {
        values.insert(key, value);
    }
}

fn normalize_legacy_option((key, value): (String, Value)) -> Option<(String, Value)> {
    match key.as_str() {
        "curr_bed_type" if value == Value::String("SuperTack Plate".to_owned()) => {
            Some((key, Value::String("Supertack Plate".to_owned())))
        }
        "enable_wipe_tower" => Some(("enable_prime_tower".to_owned(), value)),
        "wipe_tower_width" => Some(("prime_tower_width".to_owned(), value)),
        "wiping_volume" => Some(("prime_volume".to_owned(), value)),
        "wipe_tower_brim_width" => Some(("prime_tower_brim_width".to_owned(), value)),
        "tool_change_gcode" => Some(("change_filament_gcode".to_owned(), value)),
        "bridge_fan_speed" => Some(("overhang_fan_speed".to_owned(), value)),
        "infill_extruder" => Some(("sparse_infill_filament".to_owned(), value)),
        "solid_infill_extruder" => Some(("solid_infill_filament".to_owned(), value)),
        "perimeter_extruder" => Some(("wall_filament".to_owned(), value)),
        "wipe_tower_extruder" => Some(("wipe_tower_filament".to_owned(), value)),
        "support_material_extruder" => Some(("support_filament".to_owned(), value)),
        "support_material_interface_extruder" => {
            Some(("support_interface_filament".to_owned(), value))
        }
        "support_material_angle" => Some(("support_angle".to_owned(), value)),
        "support_material_enforce_layers" => Some(("enforce_support_layers".to_owned(), value)),
        "initial_layer_print_height"
        | "initial_layer_speed"
        | "internal_solid_infill_speed"
        | "top_surface_speed"
        | "outer_wall_speed"
        | "support_object_xy_distance"
            if value.as_str().is_some_and(|text| text.contains('%')) =>
        {
            None
        }
        "inherits_cummulative" => Some(("inherits_group".to_owned(), value)),
        "compatible_printers_condition_cummulative" => {
            Some(("compatible_machine_expression_group".to_owned(), value))
        }
        "compatible_prints_condition_cummulative" => {
            Some(("compatible_process_expression_group".to_owned(), value))
        }
        "cooling" => Some(("slow_down_for_layer_cooling".to_owned(), value)),
        "timelapse_no_toolhead" => Some(("timelapse_type".to_owned(), value)),
        "timelapse_type" if value == Value::String("2".to_owned()) => {
            Some((key, Value::String("0".to_owned())))
        }
        "support_type" if value == Value::String("normal".to_owned()) => {
            Some((key, Value::String("normal(manual)".to_owned())))
        }
        "support_type" if value == Value::String("tree".to_owned()) => {
            Some((key, Value::String("tree(manual)".to_owned())))
        }
        "support_type" if value == Value::String("hybrid(auto)".to_owned()) => {
            Some((key, Value::String("tree(auto)".to_owned())))
        }
        "support_base_pattern" if value == Value::String("none".to_owned()) => {
            Some((key, Value::String("hollow".to_owned())))
        }
        "different_settings_to_system" => {
            Some((key, normalize_different_settings_to_system(value)))
        }
        "overhang_fan_threshold" if value == Value::String("5%".to_owned()) => {
            Some((key, Value::String("10%".to_owned())))
        }
        "nozzle_volume_type"
        | "default_nozzle_volume_type"
        | "printer_extruder_variant"
        | "print_extruder_variant"
        | "filament_extruder_variant"
        | "extruder_variant_list" => Some((key, normalize_extruder_variant_value(value))),
        "extruder_type" => Some((
            key,
            replace_string_value(value, "DirectDrive", "Direct Drive"),
        )),
        "enable_power_loss_recovery" => Some((key, normalize_power_loss_recovery(value))),
        "ensure_vertical_shell_thickness" => Some((key, normalize_vertical_shell_thickness(value))),
        "rotate_solid_infill_direction" => Some((
            "solid_infill_rotate_template".to_owned(),
            normalize_rotate_solid_infill_direction(value),
        )),
        "sparse_infill_anchor" => Some(("infill_anchor".to_owned(), value)),
        "sparse_infill_anchor_max" => Some(("infill_anchor_max".to_owned(), value)),
        "chamber_temperatures" => Some(("chamber_temperature".to_owned(), value)),
        "thumbnail_size" => Some(("thumbnails".to_owned(), value)),
        "top_one_wall_type" if value.as_str().is_some_and(|text| text != "none") => Some((
            "only_one_wall_top".to_owned(),
            Value::Bool(true),
        )),
        "initial_layer_flow_ratio" => Some(("bottom_solid_infill_flow_ratio".to_owned(), value)),
        "ironing_direction" => Some(("ironing_angle".to_owned(), value)),
        "ironing_angle" => Some((key, normalize_legacy_ironing_angle(value))),
        "counterbole_hole_bridging" => Some(("counterbore_hole_bridging".to_owned(), value)),
        "draft_shield" if value == Value::String("limited".to_owned()) => {
            Some((key, Value::String("disabled".to_owned())))
        }
        "sparse_infill_pattern"
        | "top_surface_pattern"
        | "bottom_surface_pattern"
        | "internal_solid_infill_pattern"
        | "support_ironing_pattern" => Some((key, normalize_legacy_pattern(value))),
        "filament_map_mode" => Some((key, normalize_legacy_filament_map_mode(value))),
        "filament_type" => Some((key, normalize_legacy_filament_type(value))),
        "prime_tower_rib_wall" if value == Value::String("1".to_owned()) => Some((
            "wipe_tower_wall_type".to_owned(),
            Value::String("rib".to_owned()),
        )),
        "prime_tower_rib_wall" => None,
        "prime_tower_extra_rib_length" => Some(("wipe_tower_extra_rib_length".to_owned(), value)),
        "prime_tower_rib_width" => Some(("wipe_tower_rib_width".to_owned(), value)),
        "prime_tower_fillet_wall" => Some(("wipe_tower_fillet_wall".to_owned(), value)),
        "extruder_clearance_max_radius" => Some(("extruder_clearance_radius".to_owned(), value)),
        "machine_switch_extruder_time" => Some(("machine_tool_change_time".to_owned(), value)),
        "wall_direction" if value == Value::String("auto".to_owned()) => {
            Some((key, Value::String("ccw".to_owned())))
        }
        _ if is_obsolete_legacy_key(key.as_str()) => None,
        _ => Some((key, value)),
    }
}

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
            | "filament_prime_volume"
    )
}

fn normalize_different_settings_to_system(value: Value) -> Value {
    let Some(text) = value.as_str() else {
        return value;
    };
    let mut normalized = text.to_owned();
    let unquoted = text.replace('"', "");
    let split_keys = unquoted
        .split(';')
        .filter(|key| !key.is_empty())
        .collect::<BTreeSet<_>>();
    for split_key in split_keys {
        if let Some(alias) = legacy_key_alias(split_key) {
            normalized = normalized.replace(split_key, alias);
        }
    }
    Value::String(normalized)
}

fn normalize_wall_infill_order(value: Value) -> Value {
    match value.as_str() {
        Some("inner wall/outer wall/infill" | "infill/inner wall/outer wall") => {
            Value::String("inner wall/outer wall".to_owned())
        }
        Some("outer wall/inner wall/infill" | "infill/outer wall/inner wall") => {
            Value::String("outer wall/inner wall".to_owned())
        }
        Some("inner-outer-inner wall/infill") => Value::String("inner-outer-inner wall".to_owned()),
        _ => value,
    }
}

fn wall_infill_order_is_infill_first(value: &Value) -> bool {
    matches!(
        value.as_str(),
        Some("infill/inner wall/outer wall" | "infill/outer wall/inner wall")
    )
}

fn normalize_extruder_variant_value(value: Value) -> Value {
    let value = replace_string_value(value, "Normal", "Standard");
    replace_string_value(value, "Big Traffic", "High Flow")
}

fn replace_string_value(value: Value, from: &str, to: &str) -> Value {
    let Some(text) = value.as_str() else {
        return value;
    };
    Value::String(text.replace(from, to))
}

fn normalize_power_loss_recovery(value: Value) -> Value {
    let Some(text) = value.as_str() else {
        return value;
    };
    if text == "1" || text.eq_ignore_ascii_case("true") {
        Value::String("enable".to_owned())
    } else if text == "0" || text.eq_ignore_ascii_case("false") {
        Value::String("disable".to_owned())
    } else {
        value
    }
}

fn normalize_vertical_shell_thickness(value: Value) -> Value {
    match value.as_str() {
        Some("1") => Value::String("ensure_all".to_owned()),
        Some("0") => Value::String("ensure_moderate".to_owned()),
        _ => value,
    }
}

fn normalize_rotate_solid_infill_direction(value: Value) -> Value {
    match value.as_str() {
        Some("1") => Value::String("0,90".to_owned()),
        Some("0") => Value::String("0".to_owned()),
        _ => value,
    }
}

fn normalize_legacy_ironing_angle(value: Value) -> Value {
    match value.as_str() {
        Some(text) if text.starts_with('-') => Value::String("0".to_owned()),
        _ => value,
    }
}

fn normalize_legacy_pattern(value: Value) -> Value {
    value
}

fn normalize_legacy_filament_map_mode(value: Value) -> Value {
    match value.as_str() {
        Some("Auto") => Value::String("Auto For Flush".to_owned()),
        _ => value,
    }
}

fn normalize_legacy_filament_type(value: Value) -> Value {
    let Some(text) = value.as_str() else {
        return value;
    };

    let mut rebuild_value = false;
    let types = text
        .split_terminator(';')
        .map(|token| {
            let token = strip_surrounding_quotes(token);
            if token == "ASA-Aero" {
                rebuild_value = true;
                "ASA-AERO".to_owned()
            } else {
                token.to_owned()
            }
        })
        .collect::<Vec<_>>();

    if rebuild_value {
        Value::String(
            types
                .into_iter()
                .map(|token| format!("\"{token}\""))
                .collect::<Vec<_>>()
                .join(";"),
        )
    } else {
        value
    }
}

fn strip_surrounding_quotes(token: &str) -> &str {
    if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
        &token[1..token.len() - 1]
    } else {
        token
    }
}

fn legacy_key_alias(key: &str) -> Option<&'static str> {
    match key {
        "enable_wipe_tower" => Some("enable_prime_tower"),
        "wipe_tower_width" => Some("prime_tower_width"),
        "wiping_volume" => Some("prime_volume"),
        "wipe_tower_brim_width" => Some("prime_tower_brim_width"),
        "tool_change_gcode" => Some("change_filament_gcode"),
        "bridge_fan_speed" => Some("overhang_fan_speed"),
        "infill_extruder" => Some("sparse_infill_filament"),
        "solid_infill_extruder" => Some("solid_infill_filament"),
        "perimeter_extruder" => Some("wall_filament"),
        "wipe_tower_extruder" => Some("wipe_tower_filament"),
        "support_material_extruder" => Some("support_filament"),
        "support_material_interface_extruder" => Some("support_interface_filament"),
        "support_material_angle" => Some("support_angle"),
        "support_material_enforce_layers" => Some("enforce_support_layers"),
        "inherits_cummulative" => Some("inherits_group"),
        "compatible_printers_condition_cummulative" => Some("compatible_machine_expression_group"),
        "compatible_prints_condition_cummulative" => Some("compatible_process_expression_group"),
        "cooling" => Some("slow_down_for_layer_cooling"),
        "timelapse_no_toolhead" => Some("timelapse_type"),
        _ => None,
    }
}
