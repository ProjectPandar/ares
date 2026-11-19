//! Ordered 3D hull state for Qhull 8.0.2's `Qt` path.
//!
//! Copyright (c) 1993-2020 The Geometry Center.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use super::{
    ConvexHull3d, HullError,
    precision::{Plane, Point, Precision, dot},
};
use std::collections::BTreeSet;

mod cone;
mod matching;
mod merge;
mod output;
mod partition;
mod precision_check;
mod queue;
mod repair;

type Id = usize;
const TAIL: Id = 0;

pub(super) fn triangulate(input: &[[f32; 3]]) -> Result<Option<ConvexHull3d>, HullError> {
    let points = input.iter().map(|p| p.map(f64::from)).collect();
    let Some(mut hull) = Hull::initial(points) else {
        return Ok(None);
    };
    hull.build()?;
    hull.check_precision()?;
    Ok(Some(hull.collect_mesh(input)?))
}

struct Facet {
    vertices: Vec<Id>,
    neighbors: Vec<Id>,
    ridges: Vec<Id>,
    ridge_set_exists: bool,
    plane: Plane,
    has_plane: bool,
    top: bool,
    next: Id,
    prev: Id,
    outside: Vec<Id>,
    coplanar: Vec<Id>,
    furthest: f64,
    not_furthest: bool,
    visible: bool,
    coplanar_horizon: bool,
    merge_horizon: bool,
    cycle_next: Id,
    simplicial: bool,
    replacement: Id,
    visit: usize,
    new: bool,
    flipped: bool,
    tested: bool,
    centrum: Option<Point>,
    keep_centrum: bool,
    max_outside: f64,
    new_merge: bool,
    degenerate: bool,
    redundant: bool,
}

impl Facet {
    fn push_ridge(&mut self, ridge: Id) {
        self.ridge_set_exists = true;
        self.ridges.push(ridge);
    }

    fn new(vertices: Vec<Id>, neighbors: Vec<Id>, top: bool) -> Self {
        Self {
            vertices,
            neighbors,
            ridges: Vec::new(),
            ridge_set_exists: false,
            plane: Plane {
                normal: [0.0; 3],
                offset: 0.0,
            },
            has_plane: false,
            top,
            next: TAIL,
            prev: TAIL,
            outside: Vec::new(),
            coplanar: Vec::new(),
            furthest: -f64::MAX,
            not_furthest: false,
            visible: false,
            coplanar_horizon: false,
            merge_horizon: false,
            cycle_next: TAIL,
            simplicial: true,
            replacement: TAIL,
            visit: 0,
            new: false,
            flipped: false,
            tested: false,
            centrum: None,
            keep_centrum: false,
            max_outside: 0.0,
            new_merge: false,
            degenerate: false,
            redundant: false,
        }
    }

    fn distance(&self, point: Point) -> f64 {
        self.plane.distance(point)
    }
}

struct Ridge {
    vertices: [Id; 2],
    top: Id,
    bottom: Id,
    tested: bool,
    nonconvex: bool,
}

struct Hull {
    points: Vec<Point>,
    vertices: Vec<Id>,
    incidence: Vec<BTreeSet<Id>>,
    lost_ridge: Vec<bool>,
    deleted_vertices: Vec<Id>,
    structural_merges: std::collections::VecDeque<(Id, Option<Id>)>,
    facets: Vec<Facet>,
    ridges: Vec<Ridge>,
    cursor: Id,
    new: Option<Id>,
    visible: Option<Id>,
    inside: Point,
    narrow: bool,
    find_best_new: bool,
    not_sharp: bool,
    precision: Precision,
    max_outside: f64,
    min_vertex: f64,
    wide_merge: bool,
    snapshot_outside: f64,
    merges: usize,
    visit: usize,
}

impl Hull {
    fn build(&mut self) -> Result<(), HullError> {
        while let Some((point, facet)) = self.next_furthest() {
            self.snapshot_outside = self.max_outside.max(self.precision.outer_limit());
            self.find_horizon(point, facet);
            self.make_cone(point).ok_or(HullError::Topology)?;
            self.match_cone()?;
            self.premerge().ok_or(if self.wide_merge {
                HullError::Precision
            } else {
                HullError::Topology
            })?;
            let search = if self.merges > 50
                || self
                    .suffix(self.new.unwrap_or(TAIL))
                    .any(|f| !self.facets[f].simplicial)
            {
                partition::Search::AllNew
            } else {
                partition::Search::Neighbor
            };
            self.partition_visible(search).ok_or(HullError::Topology)?;
            self.delete_visible();
            self.reset_new();
            self.find_best_new = false;
            self.not_sharp = false;
        }
        Ok(())
    }

    // qh_createsimplex and qh_initialhull. The set order is the reverse
    // of seed allocation; neighbor slot i omits vertex i.
    fn initial(points: Vec<Point>) -> Option<Self> {
        let (precision, seed) = Precision::initial(&points)?;
        let mut hull = Self {
            points,
            vertices: [TAIL].into_iter().chain(seed).collect(),
            incidence: vec![BTreeSet::new(); 5],
            lost_ridge: vec![false; 5],
            deleted_vertices: Vec::new(),
            structural_merges: Default::default(),
            facets: vec![Facet::new(Vec::new(), Vec::new(), false)],
            ridges: Vec::new(),
            cursor: TAIL,
            new: Some(TAIL),
            visible: None,
            inside: [0.0; 3],
            narrow: false,
            find_best_new: false,
            not_sharp: false,
            snapshot_outside: precision.outer_limit(),
            precision,
            max_outside: 0.0,
            min_vertex: 0.0,
            wide_merge: false,
            merges: 0,
            visit: 0,
        };
        for axis in 0..3 {
            for id in seed.into_iter().rev() {
                hull.inside[axis] += hull.points[id][axis];
            }
            hull.inside[axis] /= 4.0;
        }
        for skip in 0..4 {
            let vertices = (1..=4)
                .rev()
                .enumerate()
                .filter_map(|(i, v)| (i != skip).then_some(v))
                .collect();
            let neighbors = (1..=4).filter(|&f| f != skip + 1).collect();
            hull.push_facet(vertices, neighbors, skip % 2 == 0);
        }
        hull.set_plane(1);
        if hull.facets[1].distance(hull.inside) > hull.precision.round {
            for facet in &mut hull.facets[1..] {
                facet.top = !facet.top;
            }
            hull.set_plane(1);
        }
        for id in 2..=4 {
            hull.set_plane(id);
        }
        if hull.facets[1..]
            .iter()
            .any(|f| f.distance(hull.inside) > hull.precision.round)
        {
            for facet in &mut hull.facets[1..] {
                facet.top = !facet.top;
                facet.plane.orient_outside(hull.inside);
            }
        }
        for facet in &hull.facets[1..] {
            if facet.distance(hull.inside) >= -hull.precision.round {
                return None;
            }
            if facet
                .neighbors
                .iter()
                .any(|&n| 0.0 + dot(facet.plane.normal, hull.facets[n].plane.normal) < -0.99999999)
            {
                hull.narrow = true;
            }
        }
        hull.reset_new();
        hull.cursor = hull.first();
        hull.partition_initial(&seed);
        hull.reset_new();
        hull.cursor = hull.first();
        hull.initial_furthest();
        Some(hull)
    }

    fn vertex(&self, id: Id) -> Point {
        self.points[self.vertices[id]]
    }

    fn set_plane(&mut self, id: Id) {
        let facet = &self.facets[id];
        let vertices: [Id; 3] = facet.vertices.as_slice().try_into().unwrap();
        let plane = Plane::new(
            vertices.map(|v| self.vertex(v)),
            facet.top,
            self.inside,
            &self.precision,
        );
        self.facets[id].plane = plane;
        self.facets[id].has_plane = true;
        self.facets[id].flipped = plane.distance(self.inside) >= -self.precision.round;
    }

    fn first(&self) -> Id {
        self.facets[TAIL].next
    }

    fn suffix(&self, id: Id) -> impl Iterator<Item = Id> + '_ {
        std::iter::successors((id != TAIL).then_some(id), |&id| {
            let next = self.facets[id].next;
            (next != TAIL).then_some(next)
        })
    }

    fn push_facet(&mut self, vertices: Vec<Id>, neighbors: Vec<Id>, top: bool) -> Id {
        let id = self.facets.len();
        for &vertex in &vertices {
            self.incidence[vertex].insert(id);
        }
        let mut facet = Facet::new(vertices, neighbors, top);
        facet.new = true;
        self.facets.push(facet);
        self.append(id);
        id
    }

    fn append(&mut self, id: Id) {
        if self.new == Some(TAIL) {
            self.new = Some(id);
            if self.visible == Some(TAIL) {
                self.visible = Some(id);
            }
        }
        if self.cursor == TAIL {
            self.cursor = id;
        }
        self.insert_before(id, TAIL);
    }

    fn insert_before(&mut self, id: Id, next: Id) {
        let prev = self.facets[next].prev;
        self.facets[id].prev = prev;
        self.facets[id].next = next;
        self.facets[prev].next = id;
        self.facets[next].prev = id;
    }

    fn unlink(&mut self, id: Id) {
        let prev = self.facets[id].prev;
        let next = self.facets[id].next;
        if self.new == Some(id) {
            self.new = Some(next);
        }
        if self.visible == Some(id) {
            self.visible = Some(next);
        }
        if self.cursor == id {
            self.cursor = next;
        }
        self.facets[prev].next = next;
        self.facets[next].prev = prev;
    }

    fn reset_new(&mut self) {
        let mut facet = self.new.take().unwrap_or(TAIL);
        while facet != TAIL {
            self.facets[facet].new = false;
            facet = self.facets[facet].next;
        }
    }

    fn initial_furthest(&mut self) {
        let mut best = TAIL;
        let mut distance = -f64::MAX;
        for id in self.suffix(self.first()) {
            let f = &self.facets[id];
            if !f.outside.is_empty() && f.furthest > distance {
                distance = f.furthest;
                best = id;
            }
        }
        if best != TAIL {
            self.unlink(best);
            self.insert_before(best, self.cursor);
            self.cursor = best;
        }
    }

    fn next_furthest(&mut self) -> Option<(Id, Id)> {
        while self.cursor != TAIL {
            let facet = &mut self.facets[self.cursor];
            if facet.outside.is_empty() {
                self.cursor = facet.next;
                continue;
            }
            if self.narrow && facet.not_furthest {
                recompute_furthest(facet, &self.points);
            }
            if self.narrow && facet.furthest < self.precision.outside {
                self.cursor = facet.next;
                continue;
            }

            return facet.outside.pop().map(|point| (point, self.cursor));
        }
        None
    }
}

fn recompute_furthest(facet: &mut Facet, points: &[Point]) {
    let mut maximum = (0, -f64::MAX);
    for (i, &point) in facet.outside.iter().enumerate() {
        let dist = facet.distance(points[point]);
        if dist > maximum.1 {
            maximum = (i, dist);
        }
    }
    let point = facet.outside.swap_remove(maximum.0);
    facet.outside.push(point);
    facet.furthest = maximum.1;
    facet.not_furthest = false;
}
