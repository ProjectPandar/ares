use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_TERMINAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("staggered_inner_seams", Bool, "false",),
    definition!("standby_temperature_delta", Int, "-5",),
    definition!("start_end_points", Points, "30x-3,54x245",),
    definition!("supertack_plate_temp", Ints, "35",),
    definition!("supertack_plate_temp_initial_layer", Ints, "35",),
];
