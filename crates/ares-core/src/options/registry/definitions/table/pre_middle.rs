use super::{OptionDefinition, OptionValueKind};

pub(super) const PRE_MIDDLE_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("complete_print_exhaust_fan_speed", Ints, "80",),
    definition!("cool_plate_temp", Ints, "35",),
    definition!("cool_plate_temp_initial_layer", Ints, "35",),
    definition!("cooling_tube_length", Float, "5",),
    definition!("cooling_tube_retraction", Float, "91.5",),
    definition!("counterbore_hole_bridging", Enum, "none",),
    definition!("curr_bed_type", Enum, "Cool Plate",),
];
