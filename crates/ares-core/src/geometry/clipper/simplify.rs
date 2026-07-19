use super::{ClipOperation, ClipperError, ClipperOptions, ClosedClipper, FillRule, PathRole};
use crate::geometry::Polygon;

pub(in crate::geometry) fn simplify_polygons(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = ClosedClipper::new(ClipperOptions {
        strictly_simple: true,
        ..ClipperOptions::default()
    });
    clipper.add_closed_paths(paths, PathRole::Subject)?;
    Ok(clipper.execute_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero))
}
