mod execute;
mod expolygon;
mod generate;
mod input;
mod opening;

#[cfg(test)]
pub(super) use execute::offset_configuration_for_test;
#[cfg(test)]
pub(crate) use execute::raw_offset_open_paths;
pub(crate) use execute::{offset_open_paths, offset_paths, offset_paths_tree, raw_offset_paths};
pub(crate) use expolygon::{
    closing_ex, offset_expolygon, offset_expolygon_refs_paths, offset_expolygons,
    offset_expolygons_paths, offset_expolygons_raw, offset2_ex, offset2_ex_with_interstage,
    opening_ex,
};
#[cfg(test)]
pub(in crate::geometry) use opening::opening_path_configurations_for_test;
pub(crate) use opening::opening_paths;
#[cfg(test)]
pub(crate) use opening::opening_paths_with_interstage;

use super::{ClipperError, PolyTree};
use crate::geometry::{ExPolygon, Polygon};

type PathsOffsetFn = fn(&[Polygon], f32, JoinType, f64) -> Result<Vec<Polygon>, ClipperError>;
type TreeOffsetFn = fn(&[Polygon], f32, JoinType, f64) -> Result<PolyTree, ClipperError>;
type ExPolygonOffsetFn = fn(&ExPolygon, f32, JoinType, f64) -> Result<Vec<ExPolygon>, ClipperError>;
type ExPolygonsOffsetFn =
    fn(&[ExPolygon], f32, JoinType, f64) -> Result<Vec<ExPolygon>, ClipperError>;
type ExPolygonsRawFn =
    fn(&[ExPolygon], f32, JoinType, f64) -> Result<(Vec<Polygon>, usize), ClipperError>;
type ExPolygonsPathsFn = fn(&[ExPolygon], f32, JoinType, f64) -> Result<Vec<Polygon>, ClipperError>;
type OffsetTwoFn =
    fn(&[ExPolygon], f32, f32, JoinType, f64) -> Result<Vec<ExPolygon>, ClipperError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JoinType {
    Square,
    Round,
    Miter,
}

struct OffsetPath {
    contour: Polygon,
    join_type: JoinType,
    end_type: EndType,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum EndType {
    ClosedPolygon,
    ClosedLine,
    OpenButt,
    OpenRound,
}

pub(crate) struct ClipperOffset {
    miter_limit: f64,
    arc_tolerance: f64,
    shortest_edge_length: f64,
    paths: Vec<OffsetPath>,
    lowest: Option<(usize, usize)>,
}

impl Default for ClipperOffset {
    fn default() -> Self {
        Self {
            miter_limit: 2.0,
            arc_tolerance: 0.25,
            shortest_edge_length: 0.0,
            paths: Vec::new(),
            lowest: None,
        }
    }
}

impl ClipperOffset {
    pub(crate) const fn miter_limit(&self) -> f64 {
        self.miter_limit
    }

    pub(crate) const fn arc_tolerance(&self) -> f64 {
        self.arc_tolerance
    }

    pub(crate) const fn shortest_edge_length(&self) -> f64 {
        self.shortest_edge_length
    }

    pub(crate) fn set_miter_limit(&mut self, value: f64) {
        self.miter_limit = value;
    }

    pub(crate) fn set_arc_tolerance(&mut self, value: f64) {
        self.arc_tolerance = value;
    }

    pub(crate) fn set_shortest_edge_length(&mut self, value: f64) {
        self.shortest_edge_length = value;
    }

    pub(crate) fn clear(&mut self) {
        self.paths.clear();
        self.lowest = None;
    }
}

const _: [JoinType; 3] = [JoinType::Square, JoinType::Round, JoinType::Miter];
const _: fn() -> ClipperOffset = ClipperOffset::default;
const _: fn(&ClipperOffset) -> f64 = ClipperOffset::miter_limit;
const _: fn(&ClipperOffset) -> f64 = ClipperOffset::arc_tolerance;
const _: fn(&ClipperOffset) -> f64 = ClipperOffset::shortest_edge_length;
const _: fn(&mut ClipperOffset, f64) = ClipperOffset::set_miter_limit;
const _: fn(&mut ClipperOffset, f64) = ClipperOffset::set_arc_tolerance;
const _: fn(&mut ClipperOffset, f64) = ClipperOffset::set_shortest_edge_length;
const _: fn(&mut ClipperOffset) = ClipperOffset::clear;
const _: fn(&mut ClipperOffset, &Polygon, JoinType) = ClipperOffset::add_closed_path;
const _: fn(&mut ClipperOffset, &[Polygon], JoinType) = ClipperOffset::add_closed_paths;
const _: fn(&mut ClipperOffset, &Polygon, JoinType) = ClipperOffset::add_closed_line;
const _: fn(&mut ClipperOffset, &Polygon, JoinType) = ClipperOffset::add_open_path;
const _: fn(&mut ClipperOffset, &Polygon, JoinType) = ClipperOffset::add_open_round_path;
const _: fn(&mut ClipperOffset, f64) -> Vec<Polygon> = ClipperOffset::generate_raw;
const _: fn(&mut ClipperOffset, f64) -> Result<Vec<Polygon>, ClipperError> =
    ClipperOffset::execute_paths;
const _: fn(&mut ClipperOffset, f64) -> Result<PolyTree, ClipperError> =
    ClipperOffset::execute_polytree;
const _: PathsOffsetFn = raw_offset_paths;
const _: PathsOffsetFn = offset_paths;
const _: TreeOffsetFn = offset_paths_tree;
const _: ExPolygonOffsetFn = offset_expolygon;
const _: ExPolygonsOffsetFn = offset_expolygons;
const _: ExPolygonsOffsetFn = closing_ex;
const _: ExPolygonsRawFn = offset_expolygons_raw;
const _: ExPolygonsPathsFn = offset_expolygons_paths;
const _: OffsetTwoFn = offset2_ex;
