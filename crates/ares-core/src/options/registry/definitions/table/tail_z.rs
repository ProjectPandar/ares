use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_Z_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "z_hop",
        Floats,
        "0.4",
        "PrintConfig.hpp:1375; PrintConfig.cpp:5122-5131",
    ),
    definition!(
        "z_hop_types",
        Enums,
        "Slope Lift",
        "PrintConfig.hpp:382-388; PrintConfig.hpp:1377; PrintConfig.cpp:526-532; PrintConfig.cpp:5149-5162",
    ),
    definition!(
        "z_offset",
        Float,
        "0",
        "PrintConfig.hpp:1609; PrintConfig.cpp:5893-5901",
    ),
    definition!(
        "zaa_dont_alternate_fill_direction",
        Bool,
        "false",
        "PrintConfig.hpp:1238; PrintConfig.cpp:4277-4282",
    ),
    definition!(
        "zaa_enabled",
        Bool,
        "false",
        "PrintConfig.hpp:1237; PrintConfig.cpp:4258-4263",
    ),
    definition!(
        "zaa_min_z",
        Float,
        "0.05",
        "PrintConfig.hpp:1239; PrintConfig.cpp:4284-4293",
    ),
    definition!(
        "zaa_minimize_perimeter_height",
        Float,
        "35",
        "PrintConfig.hpp:1240; PrintConfig.cpp:4265-4275",
    ),
];
