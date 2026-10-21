//! Point.hpp:506–637 (active ClosestPointInRadiusLookup insert/find/find_all).

use super::math::{dot, rounded, vector};
use crate::geometry::Point;
use std::collections::HashMap;

pub(super) struct Sample {
    pub(super) exact: [f64; 2],
    pub(super) contour: usize,
    pub(super) index: usize,
}

pub(super) struct Lookup {
    radius: i64,
    resolution: i64,
    shift: u32,
    // One key owns ALL equal-range values, not one overwriting PointHashMap entry.
    buckets: HashMap<(i64, i64), Vec<Sample>>,
}
impl Lookup {
    pub(super) fn new(radius: i64) -> Self {
        let grid = 2 * radius + 4;
        let shift = i64::BITS - grid.leading_zeros();
        Self {
            radius,
            resolution: 1 << shift,
            shift,
            buckets: HashMap::new(),
        }
    }
    fn key(&self, p: Point) -> (i64, i64) {
        (p.x() >> self.shift, p.y() >> self.shift)
    }
    pub(super) fn insert(&mut self, value: Sample) {
        let key = self.key(rounded(value.exact));
        self.buckets.entry(key).or_default().push(value);
    }
    fn candidates(&self, query: Point) -> impl Iterator<Item = &Sample> {
        let corner = self.key(Point::new(
            query.x() + (self.resolution >> 1),
            query.y() + (self.resolution >> 1),
        ));
        (-1..1).flat_map(move |y| {
            (-1..1).flat_map(move |x| {
                self.buckets
                    .get(&(corner.0 + x, corner.1 + y))
                    .into_iter()
                    // libstdc++ non-hinted unordered_multimap::emplace inserts before
                    // the first equivalent node; rehash preserves equivalent order.
                    .flat_map(|values| values.iter().rev())
            })
        })
    }
    pub(super) fn find(&self, query: Point) -> Option<(&Sample, f64)> {
        let mut best = None;
        let mut distance = f64::MAX;
        for value in self.candidates(query) {
            let delta = vector(rounded(value.exact), query);
            let d2 = dot(delta, delta);
            if d2 < distance {
                distance = d2;
                best = Some(value);
            }
        }
        best.filter(|_| distance < self.radius as f64 * self.radius as f64)
            .map(|v| (v, distance))
    }
    pub(super) fn find_all(&self, query: Point) -> Vec<(&Sample, f64)> {
        let r2 = self.radius as f64 * self.radius as f64;
        self.candidates(query)
            .filter_map(|v| {
                let d = vector(rounded(v.exact), query);
                let d2 = dot(d, d);
                (d2 <= r2).then_some((v, d2))
            })
            .collect()
    }
}
