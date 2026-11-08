use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_FINAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("small_support_perimeter_speed", FloatsOrPercents, "50%"; true, false, "50%"),
    definition!("small_support_perimeter_threshold", Floats, "0"; true, false, "0"),
    definition!("solid_infill_direction", Float, "45"; true, false, "45"),
    definition!("solid_infill_rotate_template", String, ""; true, false, ""),
    definition!("sparse_infill_acceleration", FloatsOrPercents, "100%"; true, false, "100%"),
    definition!("sparse_infill_density", Percent, "20"; true, false, "20%"),
    definition!("sparse_infill_filament_id", Int, "0"; true, false, "0"),
    definition!("sparse_infill_flow_ratio", Float, "1"; true, false, "1"),
    definition!("sparse_infill_line_width", FloatOrPercent, "0"; true, false, "0"),
    definition!(
        "sparse_infill_pattern",
        Enum,
        "crosshatch",
        super::INFILL_PATTERN_TYPE_KEYS; true, false, "crosshatch"),
    definition!("sparse_infill_rotate_template", String, ""; true, false, ""),
    definition!("sparse_infill_smooth_factor", Percent, "0%"; true, false, "0%"),
    definition!("sparse_infill_speed", FloatsNullable, "100"; true, false, "100"),
    definition!("spiral_finishing_flow_ratio", Float, "0"; true, false, "0"),
    definition!("spiral_mode", Bool, "false"; true, false, "0"),
    definition!("spiral_mode_max_xy_smoothing", FloatOrPercent, "200%"; true, false, "200%"),
    definition!("spiral_mode_smooth", Bool, "false"; true, false, "0"),
    definition!("spiral_starting_flow_ratio", Float, "0"; true, false, "0"),
];
