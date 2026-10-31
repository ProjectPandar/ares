// ClipperUtils.cpp:1036–1118: open clipping and closed-path recombination.
use super::{ClipOperation, ClipperError, backend};
use crate::geometry::{ExPolygon, Polygon, Polyline};

pub(crate) fn intersection_pl(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    closed(subject, clip, ClipOperation::Intersection)
}
pub(crate) fn diff_pl(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    closed(subject, clip, ClipOperation::Difference)
}
pub(crate) fn difference_open_polylines(
    subject: &[Polyline],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    backend::clip_open(subject, clip, ClipOperation::Difference)
}
pub(crate) fn intersection_open_polylines(
    subject: &[Polyline],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    backend::clip_open(subject, clip, ClipOperation::Intersection)
}
pub(crate) fn open_polyline_inside_expolygon(
    subject: &Polyline,
    clip: &ExPolygon,
) -> Result<bool, ClipperError> {
    let paths: Vec<_> = std::iter::once(clip.contour())
        .chain(clip.holes())
        .cloned()
        .collect();
    Ok(backend::clip_open(
        std::slice::from_ref(subject),
        &paths,
        ClipOperation::Difference,
    )?
    .is_empty())
}
fn closed(
    subject: &[Polygon],
    clip: &[Polygon],
    kind: ClipOperation,
) -> Result<Vec<Polyline>, ClipperError> {
    let paths: Vec<_> = subject.iter().map(Polygon::split_at_first_point).collect();
    let mut result = backend::clip_open(&paths, clip, kind)?;
    recombine_polylines(&mut result);
    Ok(result)
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
