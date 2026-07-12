use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("printhost_user", String, "", "PrintConfig.cpp:849-854",),
    definition!(
        "printing_by_object_gcode",
        String,
        "",
        "PrintConfig.hpp:1295; PrintConfig.cpp:1949-1956",
    ),
    definition!(
        "process_change_extrusion_role_gcode",
        String,
        "",
        "PrintConfig.hpp:1394; PrintConfig.cpp:4948-4955",
    ),
    definition!(
        "purge_in_prime_tower",
        Bool,
        "true",
        "PrintConfig.hpp:1458; PrintConfig.cpp:5832-5836",
    ),
];
