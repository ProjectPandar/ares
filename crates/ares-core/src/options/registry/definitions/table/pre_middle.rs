use super::{OptionDefinition, OptionValueKind};

pub(super) const PRE_MIDDLE_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("complete_print_exhaust_fan_speed", Ints, "80"; true, false, "80"),
    definition!("cool_plate_temp", Ints, "35"; true, false, "35"),
    definition!("cool_plate_temp_initial_layer", Ints, "35"; true, false, "35"),
    definition!("cooling_filter_enabled", Bool, "false"; true, false, "0"),
    definition!("cooling_tube_length", Float, "5"; true, false, "5"),
    definition!("cooling_tube_retraction", Float, "91.5"; true, false, "91.5"),
    definition!("counterbore_hole_bridging", Enum, "none"; true, false, "none"),
    definition!(
        "curr_bed_type",
        Enum,
        "Cool Plate",
        super::BED_TYPE_TYPE_KEYS; true, false, "Cool Plate"),
];
