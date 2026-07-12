use crate::{InfillOptions, options::InfillLayerRole};

pub(super) fn reverse_segment(
    role: InfillLayerRole,
    options: &InfillOptions,
    already_reversed: bool,
) -> bool {
    already_reversed
        ^ (role == InfillLayerRole::TopSurface && options.calib_flowrate_topinfill_special_order())
}
