use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_MATERIAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("material_colour", String, "#29B2B2",),
    definition!("material_correction", Floats, "1",),
    definition!("material_correction_x", Float, "1",),
    definition!("material_correction_y", Float, "1",),
    definition!("material_correction_z", Float, "1",),
    definition!("material_density", Float, "1",),
    definition!("material_print_speed", Enum, "fast",),
    definition!("material_type", String, "Tough",),
    definition!("material_vendor", String, ""),
];
