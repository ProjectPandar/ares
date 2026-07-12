use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_RAFT_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "raft_contact_distance",
        Float,
        "0.1",
        "PrintConfig.hpp:939; PrintConfig.cpp:4988-4997",
    ),
    definition!(
        "raft_expansion",
        Float,
        "1.5",
        "PrintConfig.hpp:940; PrintConfig.cpp:4999-5006",
    ),
    definition!(
        "raft_first_layer_density",
        Percent,
        "90",
        "PrintConfig.hpp:941; PrintConfig.cpp:5008-5016",
    ),
    definition!(
        "raft_first_layer_expansion",
        Float,
        "2.0",
        "PrintConfig.hpp:942; PrintConfig.cpp:5018-5026",
    ),
    definition!(
        "raft_layers",
        Int,
        "0",
        "PrintConfigConstants.hpp:6; PrintConfig.hpp:943; PrintConfig.cpp:5028-5037",
    ),
    definition!(
        "reduce_crossing_wall",
        Bool,
        "false",
        "PrintConfig.hpp:1479; PrintConfigConstants.hpp:7; PrintConfig.cpp:904-909",
    ),
    definition!(
        "reduce_fan_stop_start_freq",
        Bools,
        "false",
        "PrintConfig.hpp:1519; PrintConfig.cpp:2334-2338",
    ),
    definition!(
        "reduce_infill_retraction",
        Bool,
        "false",
        "PrintConfig.hpp:1544; PrintConfig.cpp:4829-4835",
    ),
    definition!(
        "relative_correction",
        Floats,
        "1",
        "PrintConfig.hpp:1837; PrintConfig.cpp:7312-7318",
    ),
    definition!(
        "relative_correction_x",
        Float,
        "1",
        "PrintConfig.hpp:1838; PrintConfig.cpp:7320-7326",
    ),
    definition!(
        "relative_correction_y",
        Float,
        "1",
        "PrintConfig.hpp:1839; PrintConfig.cpp:7328-7334",
    ),
    definition!(
        "relative_correction_z",
        Float,
        "1",
        "PrintConfig.hpp:1840; PrintConfig.cpp:7336-7342",
    ),
];
