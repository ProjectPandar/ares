// ClipperUtils.cpp:simplify_polygons. Clipper2 union is always strictly simple.
use super::{ClipperError, union_polygons_paths};
use crate::geometry::Polygon;

pub(in crate::geometry) fn simplify_polygons(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    union_polygons_paths(paths)
}
