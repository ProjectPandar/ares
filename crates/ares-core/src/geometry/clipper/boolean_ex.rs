use super::{
    ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, JoinType, PathRole,
    raw_offset_paths,
};
use crate::geometry::{ExPolygon, Polygon};

pub(super) const SAFETY_OFFSET: f32 = 10.0;
pub(super) const SAFETY_MITER_LIMIT: f64 = 3.0;

#[cfg(test)]
pub(in crate::geometry) fn safety_offset_configuration_for_test() -> (f64, f64) {
    super::offset::offset_configuration_for_test(SAFETY_OFFSET, JoinType::Miter, SAFETY_MITER_LIMIT)
}

pub(crate) fn difference_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    execute_ex(subject, clip, ClipOperation::Difference, PathRole::Clip)
}

pub(crate) fn difference_ex_with_safety_offset(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut expanded = Vec::new();
    for expolygon in clip {
        append_safety_offset(expolygon.contour(), &mut expanded)?;
        for hole in expolygon.holes() {
            append_safety_offset(hole, &mut expanded)?;
        }
    }
    execute_ex_with_paths(subject, &expanded, ClipOperation::Difference)
}

pub(crate) fn difference_ex_polygons(
    subject: &[ExPolygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    execute_ex_with_paths(subject, clip, ClipOperation::Difference)
}

pub(crate) fn difference_polygons_ex(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut paths_clipper = Clipper::new(ClipperOptions::default());
    paths_clipper.add_closed_paths(subject, PathRole::Subject)?;
    paths_clipper.add_closed_paths(clip, PathRole::Clip)?;
    execute_two_pass(&mut paths_clipper, ClipOperation::Difference)
}

pub(crate) fn difference_ex_polygons_with_safety_offset(
    subject: &[ExPolygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut expanded = Vec::new();
    for polygon in clip {
        append_safety_offset(polygon, &mut expanded)?;
    }
    execute_ex_with_paths(subject, &expanded, ClipOperation::Difference)
}

pub(crate) fn intersection_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    execute_ex(subject, clip, ClipOperation::Intersection, PathRole::Clip)
}

pub(crate) fn intersection_polygons_ex(
    subject: &[Polygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut paths_clipper = Clipper::new(ClipperOptions::default());
    paths_clipper.add_closed_paths(subject, PathRole::Subject)?;
    add_expolygons(&mut paths_clipper, clip, PathRole::Clip)?;
    execute_two_pass(&mut paths_clipper, ClipOperation::Intersection)
}

pub(crate) fn union_expolygons(
    current: &[ExPolygon],
    candidate: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    execute_ex(current, candidate, ClipOperation::Union, PathRole::Subject)
}

pub(crate) fn xor_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    execute_ex(subject, clip, ClipOperation::Xor, PathRole::Clip)
}

fn execute_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
    operation: ClipOperation,
    clip_role: PathRole,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut paths_clipper = Clipper::new(ClipperOptions::default());
    add_expolygons(&mut paths_clipper, subject, PathRole::Subject)?;
    add_expolygons(&mut paths_clipper, clip, clip_role)?;
    execute_two_pass(&mut paths_clipper, operation)
}

fn execute_ex_with_paths(
    subject: &[ExPolygon],
    clip: &[Polygon],
    operation: ClipOperation,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut paths_clipper = Clipper::new(ClipperOptions::default());
    add_expolygons(&mut paths_clipper, subject, PathRole::Subject)?;
    paths_clipper.add_closed_paths(clip, PathRole::Clip)?;
    execute_two_pass(&mut paths_clipper, operation)
}

fn execute_two_pass(
    paths_clipper: &mut Clipper,
    operation: ClipOperation,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let paths = paths_clipper.execute_paths(operation, FillRule::NonZero, FillRule::NonZero)?;
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let mut tree_clipper = Clipper::new(ClipperOptions::default());
    assert!(
        tree_clipper
            .add_closed_paths(&paths, PathRole::Subject)
            .expect("first-pass output paths must remain inside the validated Clipper range"),
        "nonempty first-pass output must contain a valid closed path"
    );
    Ok(tree_clipper
        .execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
        .into_expolygons())
}

pub(super) fn append_safety_offset(
    path: &Polygon,
    output: &mut Vec<Polygon>,
) -> Result<(), ClipperError> {
    output.append(&mut raw_offset_paths(
        std::slice::from_ref(path),
        SAFETY_OFFSET,
        JoinType::Miter,
        SAFETY_MITER_LIMIT,
    )?);
    Ok(())
}

fn add_expolygons(
    clipper: &mut Clipper,
    expolygons: &[ExPolygon],
    role: PathRole,
) -> Result<(), ClipperError> {
    for expolygon in expolygons {
        clipper.add_closed_path(expolygon.contour(), role)?;
        clipper.add_closed_paths(expolygon.holes(), role)?;
    }
    Ok(())
}
