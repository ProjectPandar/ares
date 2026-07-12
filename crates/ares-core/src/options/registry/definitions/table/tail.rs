use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("printhost_user", String, ""),
    definition!("printing_by_object_gcode", String, "",),
    definition!("process_change_extrusion_role_gcode", String, "",),
    definition!("purge_in_prime_tower", Bool, "true",),
];
