//! SeamPlacer.cpp:601–667, exact painted intervals.

use super::{Vec3, contour::PaintEdge};

pub(super) const TOLERANCE: f64 = 1e-3;

pub(super) struct ArcLoop<'a> {
    pub(super) xy: &'a [[f64; 2]],
    pub(super) arc: Vec<f64>,
    pub(super) z: f32,
}

impl<'a> ArcLoop<'a> {
    pub(super) fn new(xy: &'a [[f64; 2]], z: f32) -> Self {
        let mut arc = Vec::with_capacity(xy.len() + 1);
        arc.push(0.0);
        for i in 0..xy.len() {
            arc.push(arc[i] + norm2(sub(xy[(i + 1) % xy.len()], xy[i])).sqrt());
        }
        Self { xy, arc, z }
    }

    pub(super) fn length(&self) -> f64 {
        *self.arc.last().unwrap()
    }

    pub(super) fn point_at(&self, at: f64) -> Vec3 {
        let i = self
            .arc
            .partition_point(|&value| value <= at)
            .min(self.xy.len())
            - 1;
        let a = self.xy[i];
        let b = self.xy[(i + 1) % self.xy.len()];
        let length = self.arc[i + 1] - self.arc[i];
        let t = if length > 0.0 {
            (at - self.arc[i]) / length
        } else {
            0.0
        };
        Vec3::new(
            (a[0] + t * (b[0] - a[0])) as f32,
            (a[1] + t * (b[1] - a[1])) as f32,
            self.z,
        )
    }

    pub(super) fn intervals(&self, reach: f32, edges: &[PaintEdge]) -> Vec<Interval> {
        let count = self.xy.len();
        if count < 3 || edges.is_empty() || self.length() == 0.0 {
            return Vec::new();
        }
        let reach = f64::from(reach);
        let (min, max) = self
            .xy
            .iter()
            .fold((self.xy[0], self.xy[0]), |(min, max), point| {
                (
                    std::array::from_fn(|i| min[i].min(point[i])),
                    std::array::from_fn(|i| max[i].max(point[i])),
                )
            });
        let mut ends = Vec::new();
        for (point, inward) in edges.iter().filter_map(|edge| edge.crossing(self.z.into())) {
            if (0..2).any(|i| point[i] < min[i] - reach || point[i] > max[i] + reach) {
                continue;
            }
            let Some((position, tangent)) = self.project(point, reach) else {
                continue;
            };
            let side = dot(inward, tangent);
            if side.abs() < 1e-6 {
                return Vec::new();
            }
            ends.push((position, side > 0.0));
        }
        ends.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let count = ends.len();
        if count == 0 || count % 2 != 0 || (0..count).any(|i| ends[i].1 == ends[(i + 1) % count].1)
        {
            return Vec::new();
        }
        let first = usize::from(!ends[0].1);
        (first..first + count)
            .step_by(2)
            .map(|i| {
                let begin = ends[i % count].0;
                let mut end = ends[(i + 1) % count].0;
                if end < begin {
                    end += self.length();
                }
                Interval { begin, end }
            })
            .collect()
    }

    fn project(&self, point: [f64; 2], reach: f64) -> Option<(f64, [f64; 2])> {
        let count = self.xy.len();
        let mut best = f64::INFINITY;
        let mut closest = (0, 0.0);
        for i in 0..count {
            let a = self.xy[i];
            let d = sub(self.xy[(i + 1) % count], a);
            let length2 = norm2(d);
            let t = if length2 > 0.0 {
                (dot(sub(point, a), d) / length2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let distance = norm2(std::array::from_fn(|j| a[j] + t * d[j] - point[j]));
            if distance < best {
                best = distance;
                closest = (i, t);
            }
        }
        (best <= reach * reach).then(|| {
            let (i, t) = closest;
            let along = t * (self.arc[i + 1] - self.arc[i]);
            let position = self.arc[i] + along;
            let vertex = if along < TOLERANCE {
                i
            } else if self.arc[i + 1] - position < TOLERANCE {
                (i + 1) % count
            } else {
                return (position, self.direction(i, true));
            };
            let before = self.direction((vertex + count - 1) % count, false);
            let after = self.direction(vertex, true);
            (position, std::array::from_fn(|j| before[j] + after[j]))
        })
    }

    fn direction(&self, mut i: usize, forward: bool) -> [f64; 2] {
        let count = self.xy.len();
        for _ in 0..count {
            let d = sub(self.xy[(i + 1) % count], self.xy[i]);
            let length = norm2(d).sqrt();
            if length >= TOLERANCE {
                return [d[0] / length, d[1] / length];
            }
            i = if forward {
                (i + 1) % count
            } else {
                (i + count - 1) % count
            };
        }
        [0.0; 2]
    }
}

pub(super) struct Interval {
    pub(super) begin: f64,
    pub(super) end: f64,
}

pub(super) fn includes(ranges: &[Interval], at: f64, length: f64) -> bool {
    ranges.is_empty()
        || ranges.iter().any(|range| {
            [at, at + length]
                .into_iter()
                .any(|s| s >= range.begin - TOLERANCE && s <= range.end + TOLERANCE)
        })
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn norm2(a: [f64; 2]) -> f64 {
    dot(a, a)
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
