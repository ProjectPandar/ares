use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_AFTER_PAD_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("parking_pos_retraction", Float, "92",),
    definition!("part_cooling_fan_min_pwm", Int, "0",),
    definition!("pellet_flow_coefficient", Floats, "0.4157",),
    definition!("pellet_modded_printer", Bool, "false",),
    definition!("physical_extruder_map", Ints, "0",),
];
