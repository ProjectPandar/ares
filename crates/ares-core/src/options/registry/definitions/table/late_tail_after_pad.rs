use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_AFTER_PAD_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("parallel_printheads_bed_exclude_areas", Strings, ""; true, false, ""),
    definition!("parallel_printheads_count", Int, "1"; true, false, "1"),
    definition!("parking_pos_retraction", Float, "92"; true, false, "92"),
    definition!("part_cooling_fan_min_pwm", Int, "0"; true, false, "0"),
    definition!("pellet_flow_coefficient", Floats, "0.4157"; false, false, "0.4157"),
    definition!("pellet_modded_printer", Bool, "false"; true, false, "0"),
    definition!("physical_extruder_map", Ints, "0"; true, false, "0"),
];
