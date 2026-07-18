// BSL-1.0 rewrite of the closed-path Clipper 6.4.2 state machine vendored by
// OrcaSlicer at fixed commit 8500fcdccaa10b5099ac20d252af3a7c560046f1.

mod active_edges;
mod bounds;
mod engine;
mod horizontals;
mod input;
mod intersections;
mod minima;
mod offset;
pub(crate) mod ordering;
mod output;
mod polytree;
mod predicates;
mod types;
mod winding;

#[cfg(test)]
pub(crate) use bounds::{IntBounds, negative_outer};
pub(crate) use offset::{ClipperOffset, JoinType, offset2_ex};
#[cfg(test)]
pub(crate) use offset::{
    offset_expolygon, offset_expolygons, offset_expolygons_paths, offset_expolygons_raw,
    offset_paths, offset_paths_tree, raw_offset_paths,
};
#[cfg(test)]
pub(crate) use polytree::PolyNode;
pub(crate) use polytree::{PolyTree, union_ex};
pub(crate) use predicates::{fixed_round, point_in_polygon, slopes_equal};

use std::collections::BinaryHeap;

use super::{Point, Polygon};
use predicates::area;
use types::{
    EdgeArena, EdgeId, GhostJoin, IntersectionNode, Join, LocalMinimum, OutPointArena, OutRec,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathRole {
    Subject,
    Clip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClipOperation {
    Intersection,
    Union,
    Difference,
    Xor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FillRule {
    EvenOdd,
    NonZero,
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ClipperOptions {
    pub(crate) reverse_solution: bool,
    pub(crate) preserve_collinear: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClipperError {
    CoordinateOutOfRange,
}

pub(crate) struct ClosedClipper {
    options: ClipperOptions,
    using_polytree: bool,
    edges: EdgeArena,
    minima: Vec<LocalMinimum>,
    use_full_range: bool,
    scanbeam: BinaryHeap<i64>,
    active_edges: Option<EdgeId>,
    sorted_edges: Option<EdgeId>,
    out_recs: Vec<OutRec>,
    out_points: OutPointArena,
    joins: Vec<Join>,
    ghost_joins: Vec<GhostJoin>,
    intersections: Vec<IntersectionNode>,
}

impl ClosedClipper {
    pub(crate) fn new(options: ClipperOptions) -> Self {
        Self {
            options,
            using_polytree: false,
            edges: EdgeArena::default(),
            minima: Vec::new(),
            use_full_range: false,
            scanbeam: BinaryHeap::new(),
            active_edges: None,
            sorted_edges: None,
            out_recs: Vec::new(),
            out_points: OutPointArena::default(),
            joins: Vec::new(),
            ghost_joins: Vec::new(),
            intersections: Vec::new(),
        }
    }
}

const _: [PathRole; 2] = [PathRole::Subject, PathRole::Clip];
const _: [ClipOperation; 4] = [
    ClipOperation::Intersection,
    ClipOperation::Union,
    ClipOperation::Difference,
    ClipOperation::Xor,
];
const _: [FillRule; 4] = [
    FillRule::EvenOdd,
    FillRule::NonZero,
    FillRule::Positive,
    FillRule::Negative,
];
const _: [JoinType; 3] = [JoinType::Square, JoinType::Round, JoinType::Miter];
const _: fn() -> ClipperOffset = ClipperOffset::default;
const _: fn(ClipperOptions) -> ClosedClipper = ClosedClipper::new;
const _: fn(&mut ClosedClipper, &Polygon, PathRole) -> Result<bool, ClipperError> =
    ClosedClipper::add_closed_path;
const _: fn(&mut ClosedClipper, &[Polygon], PathRole) -> Result<bool, ClipperError> =
    ClosedClipper::add_closed_paths;
const _: fn(&mut ClosedClipper, ClipOperation, FillRule, FillRule) -> Vec<Polygon> =
    ClosedClipper::execute_paths;
const _: fn(&mut ClosedClipper, ClipOperation, FillRule, FillRule) -> PolyTree =
    ClosedClipper::execute_polytree;
const _: fn(&[Polygon], FillRule) -> Result<Vec<super::ExPolygon>, ClipperError> = union_ex;
const _: fn(&mut ClosedClipper) = ClosedClipper::clear;
const _: fn(f64) -> i64 = fixed_round;
const _: fn(i64, i64, i64, i64, bool) -> bool = slopes_equal;
const _: fn(Point, &[Point]) -> i32 = point_in_polygon;
const _: fn(&[Point]) -> f64 = area;
const _: fn(&EdgeArena, types::EdgeId) -> PathRole = EdgeArena::role;
