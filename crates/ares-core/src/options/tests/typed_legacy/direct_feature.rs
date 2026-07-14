use super::rule;
use crate::options::typed_legacy::{JsonArrayAllowance, LegacyAction, VectorType};

#[test]
fn typed_legacy_direct_renames_have_exact_targets_and_array_contracts() {
    let expected = [
        ("enable_wipe_tower", "enable_prime_tower", None),
        ("wipe_tower_width", "prime_tower_width", None),
        ("wiping_volume", "prime_volume", None),
        ("wipe_tower_brim_width", "prime_tower_brim_width", None),
        ("tool_change_gcode", "change_filament_gcode", None),
        ("bridge_fan_speed", "overhang_fan_speed", Some(VectorType::Ints)),
        ("wipe_tower_extruder", "wipe_tower_filament", None),
        ("support_material_extruder", "support_filament", None),
        ("support_material_interface_extruder", "support_interface_filament", None),
        ("support_material_angle", "support_angle", None),
        ("support_material_enforce_layers", "enforce_support_layers", None),
        ("cooling", "slow_down_for_layer_cooling", Some(VectorType::Bools)),
        ("timelapse_no_toolhead", "timelapse_type", None),
        ("sparse_infill_anchor", "infill_anchor", None),
        ("sparse_infill_anchor_max", "infill_anchor_max", None),
        ("chamber_temperatures", "chamber_temperature", Some(VectorType::Ints)),
        ("thumbnail_size", "thumbnails", None),
        ("initial_layer_flow_ratio", "bottom_solid_infill_flow_ratio", None),
        ("ironing_direction", "ironing_angle", None),
        ("counterbole_hole_bridging", "counterbore_hole_bridging", None),
        ("prime_tower_extra_rib_length", "wipe_tower_extra_rib_length", None),
        ("prime_tower_rib_width", "wipe_tower_rib_width", None),
        ("prime_tower_fillet_wall", "wipe_tower_fillet_wall", None),
        ("extruder_clearance_max_radius", "extruder_clearance_radius", None),
        ("machine_switch_extruder_time", "machine_tool_change_time", None),
    ];

    assert_eq!(expected.len(), 25);
    for (source, target, vector) in expected {
        assert_eq!(rule(source).action, LegacyAction::Rename { target });
        assert_eq!(
            rule(source).wire.json_array,
            vector.map_or(JsonArrayAllowance::RejectAfterFirstPass, JsonArrayAllowance::Flatten)
        );
    }
}

#[test]
fn typed_legacy_feature_filament_aliases_have_exact_inherit_parameters() {
    let expected = [
        ("infill_extruder", "sparse_infill_filament_id"),
        ("sparse_infill_filament", "sparse_infill_filament_id"),
        ("solid_infill_extruder", "internal_solid_filament_id"),
        ("solid_infill_filament", "internal_solid_filament_id"),
        ("top_solid_infill_filament", "top_surface_filament_id"),
        ("bottom_solid_infill_filament", "bottom_surface_filament_id"),
        ("perimeter_extruder", "outer_wall_filament_id"),
        ("wall_filament", "outer_wall_filament_id"),
        ("wall_filament_id", "outer_wall_filament_id"),
        ("inner_wall_filament", "inner_wall_filament_id"),
        ("outer_wall_filament", "outer_wall_filament_id"),
    ];

    assert_eq!(expected.len(), 11);
    for (source, target) in expected {
        assert_eq!(
            rule(source).action,
            LegacyAction::FeatureFilament {
                target,
                legacy_inherit: "1",
                canonical_inherit: "0",
            }
        );
        assert_eq!(rule(source).wire.json_array, JsonArrayAllowance::RejectAfterFirstPass);
    }
}

#[test]
fn typed_legacy_renames_are_single_pass_and_do_not_recurse_into_targets() {
    for source in ["ironing_direction", "timelapse_no_toolhead", "wall_filament_id"] {
        assert_eq!(
            rule(source).recursion,
            crate::options::typed_legacy::RecursionContract::SinglePass
        );
    }
}
