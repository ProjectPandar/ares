//! Fragment endpoint lookup from `Point.hpp::ClosestPointInRadiusLookup`.

use super::PathFragment;
use crate::geometry::Point;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(super) struct FragmentEnd {
    pub(super) layer: usize,
    pub(super) polyline: usize,
    pub(super) start: bool,
}

impl FragmentEnd {
    fn point(self, fragments: &[PathFragment]) -> Option<Point> {
        let line = &fragments[self.layer].polylines[self.polyline];
        if self.start {
            line.front()
        } else {
            line.back()
        }
    }
}

pub(super) struct FragmentLookup {
    cells: HashMap<(i64, i64), Vec<FragmentEnd>>,
}

impl FragmentLookup {
    pub(super) fn new(fragments: &[PathFragment]) -> Self {
        let mut cells: HashMap<_, Vec<_>> = HashMap::new();
        for (layer, polyline, _) in fragments
            .iter()
            .enumerate()
            .flat_map(|(layer, fragment)| {
                fragment
                    .polylines
                    .iter()
                    .enumerate()
                    .map(move |(polyline, line)| (layer, polyline, line))
            })
            .filter(|(_, _, line)| line.is_valid())
        {
            for start in [true, false] {
                let end = FragmentEnd {
                    layer,
                    polyline,
                    start,
                };
                let point = end.point(fragments).unwrap();
                cells
                    .entry((point.x() >> 5, point.y() >> 5))
                    .or_default()
                    .push(end);
            }
        }
        Self { cells }
    }

    pub(super) fn find(&self, point: Point, fragments: &[PathFragment]) -> Option<FragmentEnd> {
        // Radius 7 gives gridres 18, rounded to 32; four closest cells.
        let x = (point.x() + 16) >> 5;
        let y = (point.y() + 16) >> 5;
        let mut best = None;
        let mut distance = f64::MAX;
        // GNU reference unordered_multimap inserts equivalent keys first.
        for (end, candidate) in [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)]
            .iter()
            .filter_map(|cell| self.cells.get(cell))
            .flat_map(|cell| cell.iter().rev())
            .filter_map(|end| end.point(fragments).map(|candidate| (*end, candidate)))
        {
            let vx = (point.x() - candidate.x()) as f64;
            let vy = (point.y() - candidate.y()) as f64;
            let squared = vx * vx + vy * vy;
            if squared < distance {
                distance = squared;
                best = Some(end);
            }
        }
        if distance < 49.0 { best } else { None }
    }
}
