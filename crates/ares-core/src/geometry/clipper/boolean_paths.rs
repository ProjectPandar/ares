// ClipperUtils.cpp:_clipper and union_ overloads, single NonZero execution.
use super::boolean_ex::safety_offset_polygons;
use super::{
    ClipOperation, ClipperError, FillRule, JoinType, SAFETY_OFFSET, backend, offset_paths,
};
use crate::geometry::Polygon;

fn run(
    subject: &[Polygon],
    clip: &[Polygon],
    kind: ClipOperation,
) -> Result<Vec<Polygon>, ClipperError> {
    backend::clip_paths(
        backend::paths(subject),
        backend::paths(clip),
        kind,
        FillRule::NonZero,
    )
}
pub(crate) fn union_polygons_paths(paths: &[Polygon]) -> Result<Vec<Polygon>, ClipperError> {
    run(paths, &[], ClipOperation::Union)
}
pub(crate) fn union_even_odd_polygons_paths(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    backend::clip_paths(
        backend::paths(paths),
        Vec::new(),
        ClipOperation::Union,
        FillRule::EvenOdd,
    )
}
pub(crate) fn union_polygons_paths_pair(
    first: &[Polygon],
    second: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    let mut paths = backend::paths(first);
    paths.extend(backend::paths(second));
    backend::clip_paths(paths, Vec::new(), ClipOperation::Union, FillRule::NonZero)
}
pub(crate) fn intersection_polygons_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    intersection_polygons_paths_with_fill(subject, clip, FillRule::NonZero)
}
pub(crate) fn intersection_polygons_paths_with_fill(
    subject: &[Polygon],
    clip: &[Polygon],
    fill: FillRule,
) -> Result<Vec<Polygon>, ClipperError> {
    backend::clip_paths(
        backend::paths(subject),
        backend::paths(clip),
        ClipOperation::Intersection,
        fill,
    )
}
pub(crate) fn difference_polygons_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    run(subject, clip, ClipOperation::Difference)
}
pub(crate) fn intersection_polygons_paths_with_safety_offset(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    run(
        subject,
        &safety_offset_polygons(clip)?,
        ClipOperation::Intersection,
    )
}
pub(crate) fn difference_polygons_paths_with_safety_offset(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    run(
        subject,
        &safety_offset_polygons(clip)?,
        ClipOperation::Difference,
    )
}
pub(crate) fn union_safety_offset_polygons(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    offset_paths(paths, SAFETY_OFFSET, JoinType::Miter, 3.0)
}
