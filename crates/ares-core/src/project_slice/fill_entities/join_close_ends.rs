//! FillBase.cpp:213–215: base fill_surface_extrusion, before path materialization.
use crate::{
    ExtrusionRole,
    geometry::{CoordinateScale, Polyline},
};

pub(super) fn join(
    polylines: Vec<Polyline>,
    role: ExtrusionRole,
    spacing: f64,
    ordering: (bool, bool),
    scale: CoordinateScale,
) -> Vec<Polyline> {
    let (no_sort, can_reverse) = ordering;
    if role != ExtrusionRole::InternalInfill || no_sort {
        return polylines;
    }
    crate::fill::join_close_ends::join_close_ends(
        polylines,
        2.0 * spacing / scale.factor(),
        can_reverse,
        1e-4 / scale.factor(),
    )
}
