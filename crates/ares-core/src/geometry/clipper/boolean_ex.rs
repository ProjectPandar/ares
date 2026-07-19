use super::{ClipOperation, ClipperError, ClipperOptions, ClosedClipper, FillRule, PathRole};
use crate::geometry::ExPolygon;

pub(crate) fn difference_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    execute_ex(subject, clip, ClipOperation::Difference)
}

pub(crate) fn intersection_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    execute_ex(subject, clip, ClipOperation::Intersection)
}

fn execute_ex(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
    operation: ClipOperation,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut paths_clipper = ClosedClipper::new(ClipperOptions::default());
    add_expolygons(&mut paths_clipper, subject, PathRole::Subject)?;
    add_expolygons(&mut paths_clipper, clip, PathRole::Clip)?;
    let paths = paths_clipper.execute_paths(operation, FillRule::NonZero, FillRule::NonZero);
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let mut tree_clipper = ClosedClipper::new(ClipperOptions::default());
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

fn add_expolygons(
    clipper: &mut ClosedClipper,
    expolygons: &[ExPolygon],
    role: PathRole,
) -> Result<(), ClipperError> {
    for expolygon in expolygons {
        clipper.add_closed_path(expolygon.contour(), role)?;
        clipper.add_closed_paths(expolygon.holes(), role)?;
    }
    Ok(())
}
