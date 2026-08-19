use std::f64::consts::PI;

use super::word;

const MAX_ARC_DEVIATION_MM: f64 = 0.02;
const MIN_ARC_SEGMENTS_PER_SECOND: f64 = 50.0;
const MIN_ARC_SEGMENT_MM: f64 = 0.1;
const MAX_ARC_SEGMENT_MM: f64 = 2.0;

pub(super) struct ArcMotion {
    pub(super) start: [f64; 3],
    pub(super) end: [f64; 3],
    pub(super) e_delta: f64,
    pub(super) feedrate: f64,
}

pub(super) fn deltas(command: &str, code: &str, motion: ArcMotion) -> Option<Vec<[f64; 4]>> {
    let ArcMotion {
        start,
        end,
        e_delta,
        feedrate,
    } = motion;
    let i = word(code, 'I').unwrap_or(0.0);
    let j = word(code, 'J').unwrap_or(0.0);
    let radius = i.hypot(j);
    if radius <= f64::EPSILON {
        return None;
    }

    let center = [start[0] + i, start[1] + j];
    let start_radius = [start[0] - center[0], start[1] - center[1]];
    let end_radius = [end[0] - center[0], end[1] - center[1]];
    let same_xy =
        (end[0] - start[0]).abs() <= f64::EPSILON && (end[1] - start[1]).abs() <= f64::EPSILON;
    let mut sweep = if same_xy {
        0.0
    } else {
        let cross = start_radius[0] * end_radius[1] - start_radius[1] * end_radius[0];
        let dot = start_radius[0] * end_radius[0] + start_radius[1] * end_radius[1];
        let mut angle = cross.atan2(dot);
        if angle < 0.0 {
            angle += 2.0 * PI;
        }
        if command == "G2" {
            angle -= 2.0 * PI;
        }
        angle
    };
    let turns = word(code, 'P').unwrap_or(0.0);
    let turns = if same_xy && turns == 0.0 { 1.0 } else { turns } * 2.0 * PI;
    sweep += if command == "G2" { -turns } else { turns };

    let segment_mm = (8.0 * radius * MAX_ARC_DEVIATION_MM)
        .sqrt()
        .min(feedrate / MIN_ARC_SEGMENTS_PER_SECOND)
        .clamp(MIN_ARC_SEGMENT_MM, MAX_ARC_SEGMENT_MM);
    let segments = ((radius * sweep.abs() / segment_mm + 0.8) as usize).max(1);
    let mut deltas = Vec::with_capacity(segments);
    let mut previous = [start[0], start[1], start[2], 0.0];
    for segment in 1..=segments {
        let fraction = segment as f64 / segments as f64;
        let angle = sweep * fraction;
        let current = if segment == segments {
            [end[0], end[1], end[2], e_delta]
        } else {
            [
                center[0] + start_radius[0] * angle.cos() - start_radius[1] * angle.sin(),
                center[1] + start_radius[0] * angle.sin() + start_radius[1] * angle.cos(),
                start[2] + (end[2] - start[2]) * fraction,
                e_delta * fraction,
            ]
        };
        deltas.push([
            current[0] - previous[0],
            current[1] - previous[1],
            current[2] - previous[2],
            current[3] - previous[3],
        ]);
        previous = current;
    }
    Some(deltas)
}
