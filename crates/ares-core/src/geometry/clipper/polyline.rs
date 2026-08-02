use super::{ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole};
use crate::geometry::{Polygon, Polyline};

pub(crate) fn intersection_pl(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    clipper_pl_closed(ClipOperation::Intersection, subject, clip)
}

pub(crate) fn diff_pl(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    clipper_pl_closed(ClipOperation::Difference, subject, clip)
}

fn clipper_pl_closed(
    operation: ClipOperation,
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    let paths = subject
        .iter()
        .map(Polygon::split_at_first_point)
        .collect::<Vec<_>>();
    let mut result = clipper_pl_open(operation, &paths, clip)?;
    recombine_polylines(&mut result);
    Ok(result)
}

fn clipper_pl_open(
    operation: ClipOperation,
    subject: &[Polyline],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_open_paths(subject, PathRole::Subject)?;
    clipper.add_closed_paths(clip, PathRole::Clip)?;
    Ok(clipper
        .execute_polytree(operation, FillRule::NonZero, FillRule::NonZero)
        .into_open_polylines())
}

pub(crate) fn recombine_polylines(polylines: &mut Vec<Polyline>) {
    let mut i = 0;
    while i < polylines.len() {
        let mut j = i + 1;
        while j < polylines.len() {
            let i_front = polylines[i].front().expect("Clipper output is valid");
            let i_back = polylines[i].back().expect("Clipper output is valid");
            let j_front = polylines[j].front().expect("Clipper output is valid");
            let j_back = polylines[j].back().expect("Clipper output is valid");

            let branch = if i_back == j_front {
                Some((false, false))
            } else if i_front == j_back {
                Some((true, false))
            } else if i_front == j_front {
                Some((true, true))
            } else if i_back == j_back {
                Some((false, true))
            } else {
                None
            };

            let Some((prepend, reverse)) = branch else {
                j += 1;
                continue;
            };
            if reverse {
                polylines[j].reverse();
            }
            let mut joining = polylines.remove(j).into_points();
            let current =
                std::mem::replace(&mut polylines[i], Polyline::new(Vec::new())).into_points();
            let points = if prepend {
                joining.pop();
                joining.extend(current);
                joining
            } else {
                joining.remove(0);
                let mut points = current;
                points.extend(joining);
                points
            };
            polylines[i] = Polyline::new(points);
        }
        i += 1;
    }
}
