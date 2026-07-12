use super::{OptionDefinition, OptionValueKind};

pub(super) const PRE_MIDDLE_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "complete_print_exhaust_fan_speed",
        Ints,
        "80",
        "PrintConfig.cpp:1828-1835",
    ),
    definition!(
        "cool_plate_temp",
        Ints,
        "35",
        "PrintConfig.hpp:1490; PrintConfig.cpp:934-942",
    ),
    definition!(
        "cool_plate_temp_initial_layer",
        Ints,
        "35",
        "PrintConfig.hpp:1497; PrintConfig.cpp:994-1002",
    ),
    definition!(
        "cooling_tube_length",
        Float,
        "5",
        "PrintConfig.hpp:1429; PrintConfig.cpp:4787-4793",
    ),
    definition!(
        "cooling_tube_retraction",
        Float,
        "91.5",
        "PrintConfig.hpp:1428; PrintConfig.cpp:4779-4785",
    ),
    definition!(
        "counterbore_hole_bridging",
        Enum,
        "none",
        "PrintConfig.hpp:401-403; PrintConfig.hpp:1208; PrintConfig.cpp:551-556; PrintConfig.cpp:1467-1483",
    ),
    definition!(
        "curr_bed_type",
        Enum,
        "Cool Plate",
        "PrintConfig.hpp:314-323; PrintConfig.hpp:1489; PrintConfig.cpp:467-476; PrintConfig.cpp:1043-1061",
    ),
];
