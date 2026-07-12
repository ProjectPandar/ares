use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_TERMINAL_WIPE_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("wipe_tower_bridging", Float, "10",),
    definition!("wipe_tower_cone_angle", Float, "30",),
    definition!("wipe_tower_extra_flow", Percent, "100",),
    definition!("wipe_tower_extra_rib_length", Float, "0",),
    definition!("wipe_tower_extra_spacing", Percent, "100",),
    definition!("wipe_tower_filament", Int, "0",),
    definition!("wipe_tower_fillet_wall", Bool, "true",),
    definition!("wipe_tower_max_purge_speed", Float, "90",),
    definition!("wipe_tower_no_sparse_layers", Bool, "false",),
    definition!("wipe_tower_rib_width", Float, "8",),
    definition!("wipe_tower_rotation_angle", Float, "0",),
    definition!("wipe_tower_type", Enum, "type2",),
    definition!("wipe_tower_wall_type", Enum, "rib",),
    definition!("wipe_tower_x", Floats, "15",),
    definition!("wipe_tower_y", Floats, "220",),
    definition!(
        "wiping_volumes_extruders",
        Floats,
        "70,70,70,70,70,70,70,70,70,70",
    ),
    definition!("wrapping_detection_gcode", String, "",),
    definition!("wrapping_detection_layers", Int, "20",),
    definition!("wrapping_exclude_area", Points, "0x0",),
    definition!("xy_contour_compensation", Float, "0",),
    definition!("xy_hole_compensation", Float, "0",),
];
