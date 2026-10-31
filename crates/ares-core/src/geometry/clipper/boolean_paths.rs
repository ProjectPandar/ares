use super::{
    ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, JoinType, PathRole,
    boolean_ex::{SAFETY_MITER_LIMIT, SAFETY_OFFSET, safety_offset_polygons},
    offset::offset_paths,
};
use crate::geometry::{Polygon, RoundPolicy};

pub(crate) fn union_polygons_paths(paths: &[Polygon]) -> Result<Vec<Polygon>, ClipperError> {
    execute(paths, &[], ClipOperation::Union)
}

pub(crate) fn union_positive_polygons_paths<P: RoundPolicy>(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths_with_policy::<P>(paths, PathRole::Subject)?;
    clipper.execute_paths_with_policy::<P>(
        ClipOperation::Union,
        FillRule::Positive,
        FillRule::Positive,
    )
}

pub(crate) fn union_polygons_paths_pair(
    first: &[Polygon],
    second: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(first, PathRole::Subject)?;
    clipper.add_closed_paths(second, PathRole::Subject)?;
    clipper.execute_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
}

pub(crate) fn union_even_odd_polygons_paths(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(paths, PathRole::Subject)?;
    clipper.execute_paths(ClipOperation::Union, FillRule::EvenOdd, FillRule::EvenOdd)
}

pub(crate) fn union_safety_offset_polygons(
    paths: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    offset_paths(paths, SAFETY_OFFSET, JoinType::Miter, SAFETY_MITER_LIMIT)
}

pub(crate) fn intersection_polygons_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    execute(subject, clip, ClipOperation::Intersection)
}

pub(crate) fn difference_polygons_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    execute(subject, clip, ClipOperation::Difference)
}

pub(crate) fn difference_polygons_paths_with_safety_offset(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    execute(
        subject,
        &safety_offset_polygons(clip)?,
        ClipOperation::Difference,
    )
}

pub(crate) fn intersection_polygons_paths_with_safety_offset(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    execute(
        subject,
        &safety_offset_polygons(clip)?,
        ClipOperation::Intersection,
    )
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
