use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_TERMINAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "staggered_inner_seams",
        Bool,
        "false",
        "PrintConfig.hpp:945; PrintConfig.cpp:5375-5380",
    ),
    definition!(
        "standby_temperature_delta",
        Int,
        "-5",
        "PrintConfig.hpp:1565; PrintConfig.cpp:5745-5755",
    ),
    definition!(
        "start_end_points",
        Points,
        "30x-3,54x245",
        "PrintConfig.hpp:1614; PrintConfig.cpp:4821-4827",
    ),
    definition!(
        "supertack_plate_temp",
        Ints,
        "35",
        "PrintConfig.hpp:1492; PrintConfig.cpp:924-932",
    ),
    definition!(
        "supertack_plate_temp_initial_layer",
        Ints,
        "35",
        "PrintConfig.hpp:1496; PrintConfig.cpp:984-992",
    ),
];
