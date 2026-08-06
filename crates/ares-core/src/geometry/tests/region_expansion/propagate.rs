mod bounds_errors;
mod core;
mod topology;

use super::helpers::{expolygon, polygon};
use crate::geometry::WaveSeed;

pub(super) fn square_boundary() -> crate::geometry::ExPolygon {
    expolygon(&[(0, 0), (1000, 0), (1000, 1000), (0, 1000)], vec![])
}

pub(super) fn seed(src: u32, boundary: u32, points: &[(i64, i64)]) -> WaveSeed {
    WaveSeed {
        src,
        boundary,
        path: polygon(points),
    }
}
