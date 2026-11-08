//! `FillAdaptive.cpp:1287–1842`: noncrossing multiline path construction.

mod cuts;
mod junctions;
mod math;
mod paths;
mod profile;
mod sweep;
mod trim;

use crate::geometry::{BoundingBox, Line, Point, Polyline};
use math::V;

struct SweepLine {
    a: V,
    b: V,
    family: usize,
    lin: V,
    junctions: Vec<(f64, usize)>,
}

impl SweepLine {
    fn along(&self, p: V) -> f64 {
        math::family_dir(self.family).dot(p - self.a)
    }

    fn length(&self) -> f64 {
        (self.b - self.a).norm()
    }
}

struct Junction {
    p: V,
    lines: Vec<usize>,
    pairs: Vec<(usize, usize)>,
    bends: Vec<Option<(usize, usize)>>,
}

#[derive(Default)]
struct LevelPath {
    verts: Vec<V>,
    lines: Vec<usize>,
    junctions: Vec<usize>,
    turn: Vec<i32>,
    cuts: Vec<Vec<V>>,
    pushes: Vec<(V, usize)>,
    start_term: Option<usize>,
    end_term: Option<usize>,
}

#[derive(Clone, Copy)]
pub(super) struct Routing {
    pub(super) d1: f64,
    pub(super) end_overlap: f64,
    pub(super) eps: f64,
}

pub(super) fn multiline_paths(
    raw: &[Line],
    sweep: usize,
    cover: BoundingBox,
    routing: Routing,
) -> Vec<Polyline> {
    let Routing { d1, eps, .. } = routing;
    let angle = f64::from(-(sweep as i32)) * std::f64::consts::PI / 3.0;
    let Some(mut lines) = sweep::collect(raw, cover, angle, eps) else {
        return Vec::new();
    };
    let mut junctions = Vec::new();
    junctions::detect(&mut lines, &mut junctions, eps);
    junctions::shorten_dead_arms(&mut lines, &mut junctions, d1, eps);
    let mut paths = paths::trace(&lines, &mut junctions, eps);
    cuts::apply(&lines, &junctions, &mut paths, d1);
    trim::trim(&paths, &lines, &junctions, routing)
        .into_iter()
        .filter(|points| !points.is_empty())
        .map(|points| {
            Polyline::new(
                points
                    .into_iter()
                    .map(|p| {
                        let w = p.rotate(-angle);
                        Point::new(w.x.round() as i64, w.y.round() as i64)
                    })
                    .collect(),
            )
        })
        .collect()
}
