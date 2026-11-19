//! SeamPlacer.cpp:739–961, paint samples and central enforcer.

use crate::{
    SliceError,
    project_slice::seam_candidates::{PaintState, SeamCandidate, SeamCandidatePosition},
};

use super::{
    SeamPaint, Vec3, deferred,
    intervals::{ArcLoop, Interval, TOLERANCE, includes},
};

pub(in crate::project_slice::seam_placement) const OVERSAMPLING: f32 = 0.2;

pub(in crate::project_slice) fn paint_candidates(
    paint: &SeamPaint,
    vertices: Vec<SeamCandidate>,
    xy: &[[f64; 2]],
    width: f32,
) -> Result<(Vec<SeamCandidate>, bool), SliceError> {
    let loop_arc = ArcLoop::new(xy, vertices[0].position.z);
    let length = loop_arc.length();
    let green = loop_arc.intervals(width, &paint.enforcers.contour);
    let red = loop_arc.intervals(width, &paint.blockers.contour);
    // A loop with no exact green ends and all vertices near green could require
    // the full coverage proof, which is outside this slice.
    if green.is_empty()
        && vertices.len() >= 3
        && vertices
            .iter()
            .all(|vertex| paint.enforcers.near(as_vec(vertex.position), width))
    {
        return Err(deferred());
    }
    let exact = !green.is_empty() || !red.is_empty();
    let mut marks = interval_marks(&green, &red, length);
    if !red.is_empty() && green.is_empty() {
        for i in 0..vertices.len() {
            let edge = loop_arc.arc[i + 1] - loop_arc.arc[i];
            oversample(
                paint,
                loop_arc.point_at(loop_arc.arc[i]),
                edge as f32,
                |s| {
                    marks.push(loop_arc.arc[i] + f64::from(s));
                },
            );
        }
    }
    marks.iter_mut().for_each(|at| *at %= length);
    marks.sort_unstable_by(f64::total_cmp);
    let mut kept = 0;
    for i in 0..marks.len() {
        if kept == 0 || marks[i] - marks[kept - 1] >= TOLERANCE {
            marks[kept] = marks[i];
            kept += 1;
        }
    }
    marks.truncate(kept);

    let classify = |point, at| {
        if includes(&red, at, length) && paint.blockers.near(point, width) {
            PaintState::Blocked
        } else if includes(&green, at, length) && paint.enforcers.near(point, width) {
            PaintState::Enforced
        } else {
            PaintState::Neutral
        }
    };
    let perimeter_index = vertices[0].perimeter_index;
    let mut points = Vec::with_capacity(vertices.len() + marks.len());
    let inserted = |point: Vec3, at| SeamCandidate {
        position: SeamCandidatePosition {
            x: point.x,
            y: point.y,
            z: point.z,
        },
        perimeter_index,
        local_ccw_angle: 0.0,
        paint: classify(point, at),
        central_enforcer: false,
    };
    for (i, vertex) in vertices.iter().enumerate() {
        let start = as_vec(vertex.position);
        let at = loop_arc.arc[i];
        let end = loop_arc.arc[i + 1];
        points.push(SeamCandidate {
            paint: classify(start, at),
            ..*vertex
        });
        if exact {
            let first = marks.partition_point(|&value| value <= at + TOLERANCE);
            points.extend(
                marks[first..]
                    .iter()
                    .copied()
                    .take_while(|&s| s < end - TOLERANCE)
                    .map(|s| inserted(loop_arc.point_at(s), s)),
            );
        } else {
            let delta = as_vec(vertices[(i + 1) % vertices.len()].position) - start;
            let direction = delta.normalized();
            oversample(paint, start, delta.norm(), |step| {
                points.push(inserted(start + direction * step, 0.0));
            });
        }
    }
    select_center(&mut points, &loop_arc, &green);
    Ok((points, !green.is_empty()))
}

fn interval_marks(green: &[Interval], red: &[Interval], length: f64) -> Vec<f64> {
    let mut marks = Vec::new();
    for (i, range) in green.iter().enumerate() {
        let middle = (range.begin + range.end) * 0.5;
        let next = next_begin(green, i, length);
        marks.extend([range.begin, range.end, middle, (range.end + next) * 0.5]);
        let mut at = range.begin + f64::from(OVERSAMPLING);
        while at < range.end - TOLERANCE {
            marks.push(at);
            at += f64::from(OVERSAMPLING);
        }
    }
    for (i, range) in red.iter().enumerate() {
        let next = next_begin(red, i, length);
        let guard = (2.0 * TOLERANCE).min(0.25 * (next - range.end));
        marks.extend([
            range.begin,
            range.end,
            (range.begin + range.end) * 0.5,
            range.end + guard,
            next - guard,
            (range.end + next) * 0.5,
        ]);
    }
    marks
}

fn next_begin(ranges: &[Interval], i: usize, length: f64) -> f64 {
    ranges
        .get(i + 1)
        .map_or(ranges[0].begin + length, |next| next.begin)
}

fn oversample(paint: &SeamPaint, start: Vec3, length: f32, mut emit: impl FnMut(f32)) {
    if paint.enforcers.near(start, length) {
        let mut step = OVERSAMPLING;
        while step < length {
            emit(step);
            step += OVERSAMPLING;
        }
    }
}

fn select_center(points: &mut [SeamCandidate], loop_arc: &ArcLoop<'_>, green: &[Interval]) {
    let count = points.len();
    let mut longest = (0, 0);
    for i in 0..count {
        let start = (i + 1) % count;
        if points[i].paint == PaintState::Enforced || points[start].paint != PaintState::Enforced {
            continue;
        }
        let len = (0..count)
            .take_while(|offset| points[(start + offset) % count].paint == PaintState::Enforced)
            .count();
        if len > longest.1 {
            longest = (start, len);
        }
    }
    let (start, len) = longest;
    if len == 0 {
        return;
    }
    let patch = (0..len).map(|offset| (start + offset) % count);
    let sharp = patch
        .clone()
        .filter(|&i| points[i].local_ccw_angle.abs() > 55.0 * std::f32::consts::PI / 180.0)
        .collect::<Vec<_>>();
    if !sharp.is_empty() {
        points[sharp[sharp.len() / 2]].central_enforcer = true;
        return;
    }
    let middles = green
        .iter()
        .map(|range| {
            (
                loop_arc.point_at((0.5 * (range.begin + range.end)) % loop_arc.length()),
                range.end - range.begin,
            )
        })
        .collect::<Vec<_>>();
    let mut central = (start + len / 2) % count;
    let mut longest = -1.0;
    for i in patch {
        let position = as_vec(points[i].position);
        for &(middle, length) in &middles {
            if length > longest && f64::from((position - middle).norm()) <= TOLERANCE {
                central = i;
                longest = length;
            }
        }
    }
    points[central].central_enforcer = true;
}

fn as_vec(point: SeamCandidatePosition) -> Vec3 {
    Vec3::new(point.x, point.y, point.z)
}
