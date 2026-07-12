use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "make_overhang_printable",
        Bool,
        "false",
        "PrintConfig.hpp:1199; PrintConfig.cpp:4850-4855",
    ),
    definition!(
        "make_overhang_printable_angle",
        Float,
        "55",
        "PrintConfig.hpp:1032; PrintConfig.cpp:4857-4867",
    ),
    definition!(
        "make_overhang_printable_hole_size",
        Float,
        "0",
        "PrintConfig.hpp:1033; PrintConfig.cpp:4869-4877",
    ),
    definition!(
        "manual_filament_change",
        Bool,
        "false",
        "PrintConfig.hpp:1389; PrintConfig.cpp:5813-5819",
    ),
    definition!(
        "master_extruder_id",
        Int,
        "1",
        "PrintConfig.hpp:1412; PrintConfig.cpp:5266-5270",
    ),
];
