mod execute;
mod generation;
mod input;

use super::helpers::{coordinates, polygon};
use crate::geometry::clipper::{ClipperOffset, JoinType};

pub(super) fn raw_open(points: &[(i64, i64)], delta: f64) -> Vec<Vec<(i64, i64)>> {
    let mut offset = ClipperOffset::default();
    offset.add_open_round_path(&polygon(points), JoinType::Round);
    offset.generate_raw(delta).iter().map(coordinates).collect()
}

pub(super) fn raw_closed(points: &[(i64, i64)], delta: f64) -> Vec<Vec<(i64, i64)>> {
    let mut offset = ClipperOffset::default();
    offset.add_closed_line(&polygon(points), JoinType::Round);
    offset.generate_raw(delta).iter().map(coordinates).collect()
}
