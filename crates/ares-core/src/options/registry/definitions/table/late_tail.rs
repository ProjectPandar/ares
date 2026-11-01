use super::{OptionDefinition, OptionValueKind};

pub(super) const LATE_TAIL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("make_overhang_printable", Bool, "false"; true, false, "0"),
    definition!("make_overhang_printable_angle", Float, "55"; true, false, "55"),
    definition!("make_overhang_printable_hole_size", Float, "0"; true, false, "0"),
    definition!("manual_filament_change", Bool, "false"; true, false, "0"),
    definition!("master_extruder_id", Int, "1"; true, false, "1"),
];
