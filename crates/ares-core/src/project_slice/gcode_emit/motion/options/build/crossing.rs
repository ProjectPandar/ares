use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

/// Rectangle-specialized seam reached from
/// `AvoidCrossingPerimeters.cpp:1099-1347`: the safe travel contour sits one
/// perimeter spacing plus half another spacing inside the slice boundary.
pub(super) fn boundary_inset(traversal: &PreparedPostClassicTraversal) -> f64 {
    traversal
        .objects
        .first()
        .and_then(|object| {
            object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .records
                .iter()
                .flatten()
                .nth(1)
        })
        .map_or(0.0, |record| {
            1.5 * traversal.scale.unscale(record.perimeter_spacing)
        })
}
