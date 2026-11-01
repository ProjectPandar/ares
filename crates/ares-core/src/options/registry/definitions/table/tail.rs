use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!("printhost_user", String, ""; true, false, ""),
    definition!("printing_by_object_gcode", String, ""; true, false, ""),
    definition!("process_change_extrusion_role_gcode", String, ""; true, false, ""),
    definition!("purge_in_prime_tower", Bool, "true"; true, false, "1"),
];
