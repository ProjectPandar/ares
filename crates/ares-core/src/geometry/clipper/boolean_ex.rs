// ClipperUtils.cpp:_clipper_ex and its overloads. Execute directly into PolyTree64.
use super::{
    ClipOperation, ClipperError, FillRule, JoinType, backend, offset_expolygons, offset_paths,
    offset_paths_tree,
};
use crate::geometry::{ExPolygon, Polygon};

pub(crate) const SAFETY_OFFSET: f32 = 10.0;

fn run(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
    kind: ClipOperation,
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(backend::clip_tree(subject, clip, kind, FillRule::NonZero)?
        .into_expolygons(crate::geometry::CoordinateScale::Normal))
}
fn run_paths(
    subject: &[ExPolygon],
    clip: &[Polygon],
    kind: ClipOperation,
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(backend::clip_tree(subject, clip, kind, FillRule::NonZero)?
        .into_expolygons(crate::geometry::CoordinateScale::Normal))
}
fn run_polygons(
    subject: &[Polygon],
    clip: &[Polygon],
    kind: ClipOperation,
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(backend::clip_tree(subject, clip, kind, FillRule::NonZero)?
        .into_expolygons(crate::geometry::CoordinateScale::Normal))
}
pub(crate) fn difference_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run(subject, clip, ClipOperation::Difference)
}
pub(crate) fn difference_ex_refs(
    subject: &[&ExPolygon],
    clip: &[&ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(
        backend::clip_tree(subject, clip, ClipOperation::Difference, FillRule::NonZero)?
            .into_expolygons(crate::geometry::CoordinateScale::Normal),
    )
}
pub(crate) fn intersection_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run(subject, clip, ClipOperation::Intersection)
}
pub(crate) fn intersection_ex_refs(
    subject: &[&ExPolygon],
    clip: &[&ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(backend::clip_tree(
        subject,
        clip,
        ClipOperation::Intersection,
        FillRule::NonZero,
    )?
    .into_expolygons(crate::geometry::CoordinateScale::Normal))
}
pub(crate) fn intersection_ex_refs_ex(
    subject: &[&ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(backend::clip_tree(
        subject,
        clip,
        ClipOperation::Intersection,
        FillRule::NonZero,
    )?
    .into_expolygons(crate::geometry::CoordinateScale::Normal))
}
pub(crate) fn xor_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run(subject, clip, ClipOperation::Xor)
}
pub(crate) fn union_expolygons(
    first: &[ExPolygon],
    second: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(
        backend::clip_tree((first, second), (), ClipOperation::Union, FillRule::NonZero)?
            .into_expolygons(crate::geometry::CoordinateScale::Normal),
    )
}
pub(crate) fn difference_ex_polygons(
    subject: &[ExPolygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_paths(subject, clip, ClipOperation::Difference)
}
pub(crate) fn intersection_ex_polygons(
    subject: &[ExPolygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_paths(subject, clip, ClipOperation::Intersection)
}
pub(crate) fn difference_polygons_ex(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_polygons(subject, clip, ClipOperation::Difference)
}
pub(crate) fn intersection_polygons_polygons_ex(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_polygons(subject, clip, ClipOperation::Intersection)
}
pub(crate) fn intersection_polygons_ex(
    subject: &[Polygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(backend::clip_tree(
        subject,
        clip,
        ClipOperation::Intersection,
        FillRule::NonZero,
    )?
    .into_expolygons(crate::geometry::CoordinateScale::Normal))
}
pub(super) fn safety_offset_polygons(paths: &[Polygon]) -> Result<Vec<Polygon>, ClipperError> {
    offset_paths(paths, SAFETY_OFFSET, JoinType::Miter, 3.0)
}
fn safety_offset_expolygons(paths: &[ExPolygon]) -> Result<Vec<Polygon>, ClipperError> {
    super::offset::offset_raw_expolygon_paths(paths, SAFETY_OFFSET, JoinType::Miter, 3.0)
}
pub(crate) fn difference_ex_with_safety_offset(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_paths(
        subject,
        &safety_offset_expolygons(clip)?,
        ClipOperation::Difference,
    )
}
pub(crate) fn intersection_ex_with_safety_offset(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_paths(
        subject,
        &safety_offset_expolygons(clip)?,
        ClipOperation::Intersection,
    )
}
pub(crate) fn difference_ex_polygons_with_safety_offset(
    subject: &[ExPolygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_paths(
        subject,
        &safety_offset_polygons(clip)?,
        ClipOperation::Difference,
    )
}
pub(crate) fn difference_polygons_ex_with_safety_offset(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    run_polygons(
        subject,
        &safety_offset_polygons(clip)?,
        ClipOperation::Difference,
    )
}
pub(crate) fn union_safety_offset_ex(paths: &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(
        offset_paths_tree(paths, SAFETY_OFFSET, JoinType::Miter, 3.0)?
            .into_expolygons(crate::geometry::CoordinateScale::Normal),
    )
}
pub(crate) fn union_safety_offset_expolygons(
    paths: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    offset_expolygons(paths, SAFETY_OFFSET, JoinType::Miter, 3.0)
}
