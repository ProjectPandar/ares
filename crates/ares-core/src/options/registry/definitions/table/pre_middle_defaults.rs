use super::{OptionDefinition, OptionValueKind};

pub(super) const PRE_MIDDLE_DEFAULT_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "default_acceleration",
        Float,
        "500",
        "PrintConfig.cpp:1779-1786",
    ),
    definition!("default_bed_type", String, "", "PrintConfig.cpp:1065-1069",),
    definition!(
        "default_filament_colour",
        Strings,
        "",
        "PrintConfig.hpp:1331; PrintConfig.cpp:2359-2365",
    ),
    definition!(
        "default_filament_profile",
        Strings,
        "",
        "PrintConfig.cpp:1788-1792",
    ),
    definition!(
        "default_jerk",
        Float,
        "0",
        "PrintConfig.hpp:1052; PrintConfig.cpp:3169-3176",
    ),
    definition!(
        "default_junction_deviation",
        Float,
        "0",
        "PrintConfig.hpp:1060; PrintConfig.cpp:3178-3186",
    ),
    definition!(
        "default_nozzle_volume_type",
        Enums,
        "Standard",
        "PrintConfig.hpp:418-421; PrintConfig.cpp:571-575; PrintConfig.cpp:5227-5237",
    ),
    definition!(
        "default_print_profile",
        String,
        "",
        "PrintConfig.cpp:1794-1798",
    ),
    definition!(
        "default_sla_material_profile",
        String,
        "",
        "PrintConfig.cpp:7513-7517",
    ),
    definition!(
        "default_sla_print_profile",
        String,
        "",
        "PrintConfig.cpp:7525-7529",
    ),
];
