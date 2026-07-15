use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_FINAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("solid_infill_direction", Float, "45",),
    definition!("solid_infill_rotate_template", String, "",),
    definition!("sparse_infill_acceleration", FloatOrPercent, "100%",),
    definition!("sparse_infill_density", Percent, "20",),
    definition!("sparse_infill_filament_id", Int, "0",),
    definition!("sparse_infill_flow_ratio", Float, "1",),
    definition!("sparse_infill_line_width", FloatOrPercent, "0",),
    definition!("sparse_infill_pattern", Enum, "crosshatch",),
    definition!("sparse_infill_rotate_template", String, "",),
    definition!("sparse_infill_speed", Float, "100",),
    definition!("spiral_finishing_flow_ratio", Float, "0",),
    definition!("spiral_mode", Bool, "false",),
    definition!("spiral_mode_max_xy_smoothing", FloatOrPercent, "200%",),
    definition!("spiral_mode_smooth", Bool, "false",),
    definition!("spiral_starting_flow_ratio", Float, "0",),
];
