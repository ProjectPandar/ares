use super::{ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole};
use crate::geometry::Polygon;

pub(crate) fn union_polygons_paths(paths: &[Polygon]) -> Result<Vec<Polygon>, ClipperError> {
    execute(paths, &[], ClipOperation::Union)
}

pub(crate) fn intersection_polygons_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    execute(subject, clip, ClipOperation::Intersection)
}

fn execute(
    subject: &[Polygon],
    clip: &[Polygon],
    operation: ClipOperation,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(subject, PathRole::Subject)?;
    clipper.add_closed_paths(clip, PathRole::Clip)?;
    clipper.execute_paths(operation, FillRule::NonZero, FillRule::NonZero)
}
