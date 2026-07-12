use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("make_overhang_printable", Bool, "false",),
    definition!("make_overhang_printable_angle", Float, "55",),
    definition!("make_overhang_printable_hole_size", Float, "0",),
    definition!("manual_filament_change", Bool, "false",),
    definition!("master_extruder_id", Int, "1",),
];
