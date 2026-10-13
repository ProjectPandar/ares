//! Model.cpp1586-1599 / PrintObject.cpp93-130: one coordinate frame per group.
use crate::{
    ProjectObject, SliceError,
    geometry::{CoordinateScale, Point},
    project::effective_config::types::ResolvedPrintObjectConfig,
};
mod rotation;

pub(super) fn center(
    source: &ProjectObject,
    group: &ResolvedPrintObjectConfig,
    scale: CoordinateScale,
) -> Result<Point, SliceError> {
    let mut center = super::raw_intersections::raw_center_unscaled(source)?;
    let original = source
        .instances()
        .first()
        .expect("resolved source has an instance")
        .transform();
    let member = source.instances()[*group
        .instance_indices
        .first()
        .expect("print group has an instance")]
    .transform();
    let angle = rotation::difference(original, member);
    if angle.abs() > 1e-4 {
        let (s, c) = angle.sin_cos();
        center = [c * center[0] - s * center[1], s * center[0] + c * center[1]];
    }
    Ok(Point::new(
        scale
            .checked_scale(center[0])
            .ok_or_else(super::raw_intersections::coordinate_error)?,
        scale
            .checked_scale(center[1])
            .ok_or_else(super::raw_intersections::coordinate_error)?,
    ))
}
