use super::{OptionDefinition, OptionValueKind};

pub(super) const PRE_MIDDLE_DEFAULT_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("default_acceleration", Float, "500",),
    definition!("default_bed_type", String, ""),
    definition!("default_filament_colour", Strings, "",),
    definition!("default_filament_profile", Strings, "",),
    definition!("default_jerk", Float, "0",),
    definition!("default_junction_deviation", Float, "0",),
    definition!("default_nozzle_volume_type", Enums, "Standard",),
    definition!("default_print_profile", String, "",),
    definition!("default_sla_material_profile", String, "",),
    definition!("default_sla_print_profile", String, "",),
];
