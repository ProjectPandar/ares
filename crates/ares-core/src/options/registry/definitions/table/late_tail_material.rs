use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_MATERIAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("material_colour", String, "#29B2B2"; false, false, ""),
    definition!("material_correction", Floats, "1"; false, false, ""),
    definition!("material_correction_x", Float, "1"; false, false, ""),
    definition!("material_correction_y", Float, "1"; false, false, ""),
    definition!("material_correction_z", Float, "1"; false, false, ""),
    definition!("material_density", Float, "1"; false, false, ""),
    definition!("material_print_speed", Enum, "fast"; false, false, ""),
    definition!("material_type", String, "Tough"; false, false, ""),
    definition!("material_vendor", String, ""; false, false, ""),
];
