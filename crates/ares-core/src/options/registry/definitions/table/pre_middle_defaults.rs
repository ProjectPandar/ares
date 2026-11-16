use super::{OptionDefinition, OptionValueKind};

pub(super) const PRE_MIDDLE_DEFAULT_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("default_acceleration", FloatsNullable, "500"; true, false, "500"),
    definition!("default_bed_type", String, ""; false, false, ""),
    definition!("default_filament_colour", Strings, ""; true, false, "\"\""),
    definition!("default_filament_profile", Strings, ""; false, false, ""),
    definition!("default_jerk", FloatsNullable, "0"; true, false, "0"),
    definition!("default_junction_deviation", FloatsNullable, "0"; true, false, "0"),
    definition!("default_nozzle_volume_type", Enums, "Standard", crate::NozzleVolumeType::TYPE_KEYS; false, false, "Standard"),
    definition!("default_print_profile", String, ""; false, false, ""),
    definition!("default_sla_material_profile", String, ""; false, false, ""),
    definition!("default_sla_print_profile", String, ""; false, false, ""),
];
