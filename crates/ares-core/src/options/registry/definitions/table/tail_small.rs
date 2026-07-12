use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_SMALL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "small_area_infill_flow_compensation",
        Bool,
        "false",
        "PrintConfig.hpp:1211; PrintConfig.cpp:4352-4357",
    ),
    definition!(
        "small_area_infill_flow_compensation_model",
        Strings,
        "0,0\n0.2,0.4444\n0.4,0.6145\n0.6,0.7059\n0.8,0.7619\n1.5,0.8571\n2,0.8889\n3,0.9231\n5,0.9520\n10,1",
        "PrintConfig.hpp:1464; PrintConfig.cpp:4359-4371",
    ),
    definition!(
        "small_perimeter_speed",
        FloatOrPercent,
        "50%",
        "PrintConfig.hpp:1191; PrintConfig.cpp:2049-2059",
    ),
    definition!(
        "small_perimeter_threshold",
        Float,
        "0",
        "PrintConfig.hpp:1192; PrintConfig.cpp:2061-2068",
    ),
];
