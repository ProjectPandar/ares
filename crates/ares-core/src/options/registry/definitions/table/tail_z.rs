use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_Z_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("z_hop", Floats, "0.4",),
    definition!("z_hop_types", Enums, "Slope Lift",),
    definition!("z_offset", Float, "0",),
    definition!("zaa_dont_alternate_fill_direction", Bool, "false",),
    definition!("zaa_enabled", Bool, "false",),
    definition!("zaa_min_z", Float, "0.05",),
    definition!("zaa_minimize_perimeter_height", Float, "35",),
];
