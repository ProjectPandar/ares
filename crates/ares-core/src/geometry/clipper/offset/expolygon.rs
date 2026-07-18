use super::execute::{configured_offset, difference_paths, union_paths, union_tree};
use super::{JoinType, offset_paths_tree};
use crate::geometry::clipper::{ClipperError, FillRule};
use crate::geometry::{ExPolygon, Polygon};

pub(crate) fn offset_expolygon(
    expolygon: &ExPolygon,
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let (paths, _) = offset_expolygon_paths(expolygon, delta, join_type, miter_limit)?;
    Ok(union_tree(&paths, FillRule::EvenOdd)?.into_expolygons())
}

pub(crate) fn offset_expolygons(
    expolygons: &[ExPolygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let (paths, _) = offset_expolygons_raw(expolygons, delta, join_type, miter_limit)?;
    Ok(union_tree(&paths, FillRule::NonZero)?.into_expolygons())
}

pub(crate) fn offset2_ex(
    expolygons: &[ExPolygon],
    first: f32,
    second: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let first_stage = offset_expolygons_paths(expolygons, first, join_type, miter_limit)?;
    Ok(offset_paths_tree(&first_stage, second, join_type, miter_limit)?.into_expolygons())
}

pub(crate) fn offset_expolygons_raw(
    expolygons: &[ExPolygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<(Vec<Polygon>, usize), ClipperError> {
    let mut output = Vec::with_capacity(expolygons.len());
    let mut collected = 0;
    for expolygon in expolygons {
        let (mut paths, survives) =
            offset_expolygon_paths(expolygon, delta, join_type, miter_limit)?;
        output.append(&mut paths);
        collected += usize::from(survives);
    }
    Ok((output, collected))
}

pub(crate) fn offset_expolygons_paths(
    expolygons: &[ExPolygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let (paths, collected) = offset_expolygons_raw(expolygons, delta, join_type, miter_limit)?;
    if collected > 1 && delta > 0.0 {
        union_paths(&paths, FillRule::NonZero)
    } else {
        Ok(paths)
    }
}

fn offset_expolygon_paths(
    expolygon: &ExPolygon,
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<(Vec<Polygon>, bool), ClipperError> {
    let mut contour_offset = configured_offset(delta, join_type, miter_limit);
    contour_offset.add_closed_path(expolygon.contour(), join_type);
    let mut contours = contour_offset.execute_paths(f64::from(delta))?;
    if contours.is_empty() {
        return Ok((Vec::new(), false));
    }
    if expolygon.holes().is_empty() {
        return Ok((contours, true));
    }

    let mut holes = Vec::with_capacity(expolygon.holes().len());
    for hole in expolygon.holes() {
        let mut hole_offset = configured_offset(delta, join_type, miter_limit);
        hole_offset.add_closed_path(hole, join_type);
        holes.append(&mut hole_offset.execute_paths(f64::from(-delta))?);
    }

    if holes.is_empty() {
        Ok((contours, true))
    } else if delta < 0.0 {
        let output = difference_paths(&contours, &holes)?;
        let survives = !output.is_empty();
        Ok((output, survives))
    } else {
        for hole in &mut holes {
            hole.reverse();
        }
        contours.append(&mut holes);
        Ok((contours, true))
    }
}
