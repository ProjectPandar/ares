use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_AFTER_PAD_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "parking_pos_retraction",
        Float,
        "92",
        "PrintConfig.hpp:1431; PrintConfig.cpp:4803-4810",
    ),
    definition!(
        "part_cooling_fan_min_pwm",
        Int,
        "0",
        "PrintConfig.hpp:1316; PrintConfig.cpp:3740-3760",
    ),
    definition!(
        "pellet_flow_coefficient",
        Floats,
        "0.4157",
        "PrintConfig.cpp:2551-2555",
    ),
    definition!(
        "pellet_modded_printer",
        Bool,
        "false",
        "PrintConfig.cpp:3819-3823",
    ),
    definition!(
        "physical_extruder_map",
        Ints,
        "0",
        "PrintConfig.hpp:1341; PrintConfig.cpp:2407-2412",
    ),
];
