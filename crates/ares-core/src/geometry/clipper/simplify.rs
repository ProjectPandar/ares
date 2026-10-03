use super::{ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole};
use crate::geometry::Polygon;

pub(in crate::geometry) fn simplify_polygons(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    simplify_polygons_with_mode(paths, true)
}

pub(in crate::geometry) fn simplify_polygons_with_mode(
    paths: &[Polygon],
    strictly_simple: bool,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions {
        strictly_simple,
        ..ClipperOptions::default()
    });
    clipper.add_closed_paths(paths, PathRole::Subject)?;
    clipper.execute_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
}
