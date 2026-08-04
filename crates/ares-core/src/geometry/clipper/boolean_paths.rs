use super::{
    ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole,
    boolean_ex::append_safety_offset,
};
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

pub(crate) fn difference_polygons_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    execute(subject, clip, ClipOperation::Difference)
}

pub(crate) fn intersection_polygons_paths_with_safety_offset(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    execute(
        subject,
        &safety_offset_clip_paths(clip)?,
        ClipOperation::Intersection,
    )
}

fn safety_offset_clip_paths(clip: &[Polygon]) -> Result<Vec<Polygon>, ClipperError> {
    let mut expanded = Vec::new();
    for path in clip {
        append_safety_offset(path, &mut expanded)?;
    }
    Ok(expanded)
}

#[cfg(test)]
pub(crate) fn safety_offset_clip_paths_for_test(
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    safety_offset_clip_paths(clip)
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
