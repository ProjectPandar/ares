use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_MATERIAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "material_colour",
        String,
        "#29B2B2",
        "PrintConfig.cpp:7372-7376",
    ),
    definition!(
        "material_correction",
        Floats,
        "1",
        "PrintConfig.hpp:1817; PrintConfig.cpp:7479-7484",
    ),
    definition!(
        "material_correction_x",
        Float,
        "1",
        "PrintConfig.hpp:1818; PrintConfig.cpp:7486-7491",
    ),
    definition!(
        "material_correction_y",
        Float,
        "1",
        "PrintConfig.hpp:1819; PrintConfig.cpp:7493-7498",
    ),
    definition!(
        "material_correction_z",
        Float,
        "1",
        "PrintConfig.hpp:1820; PrintConfig.cpp:7500-7505",
    ),
    definition!(
        "material_density",
        Float,
        "1",
        "PrintConfig.hpp:1814; PrintConfig.cpp:7411-7416",
    ),
    definition!(
        "material_print_speed",
        Enum,
        "fast",
        "PrintConfig.hpp:1805; PrintConfig.hpp:1821; PrintConfig.cpp:413-417; PrintConfig.cpp:7855-7864",
    ),
    definition!(
        "material_type",
        String,
        "Tough",
        "PrintConfig.cpp:7378-7388",
    ),
    definition!("material_vendor", String, "", "PrintConfig.cpp:7507-7511",),
];
