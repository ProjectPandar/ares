use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_RAFT_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("raft_contact_distance", Float, "0.1",),
    definition!("raft_expansion", Float, "1.5",),
    definition!("raft_first_layer_density", Percent, "90",),
    definition!("raft_first_layer_expansion", Float, "2.0",),
    definition!("raft_layers", Int, "0",),
    definition!("reduce_crossing_wall", Bool, "false",),
    definition!("reduce_fan_stop_start_freq", Bools, "false",),
    definition!("reduce_infill_retraction", Bool, "false",),
    definition!("relative_correction", Floats, "1",),
    definition!("relative_correction_x", Float, "1",),
    definition!("relative_correction_y", Float, "1",),
    definition!("relative_correction_z", Float, "1",),
];
