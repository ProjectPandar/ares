use crate::geometry::Coord;

use super::super::traversal::PreparedPostClassicTraversal;

pub(in crate::project_slice) struct PreparedPostClassicRawPaths {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedRawPathObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedRawPathObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedRawPathRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedRawPathRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedRawPathSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedRawPathSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) roots: Vec<RawPathNode>,
    /// Root-level chain over the compacted loops plus `thin_entities`
    /// (`PerimeterGenerator.cpp:238`), computed once during materialization
    /// and consumed verbatim by the emission traversal — the single chain
    /// computation for the surface level.
    pub(in crate::project_slice) root_order: Vec<(usize, bool)>,
    /// `variable_width` thin-wall entities (`PerimeterGenerator.cpp:231`),
    /// materialized once here because the root chain interleaves them.
    pub(in crate::project_slice) thin_entities:
        Vec<crate::project_slice::perimeters::classic::entity_collections::ExtrusionEntity>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct RawPathNode {
    pub(in crate::project_slice) paths: Vec<ExtrusionPath>,
    pub(in crate::project_slice) children: Vec<RawPathNode>,
    /// Chain over this node's children level (`PerimeterGenerator.cpp:238`),
    /// computed once during materialization and consumed verbatim by the
    /// emission traversal's child descent and ordering. Loop-only level:
    /// reverse flags stay `false` (chain-extrusion loop semantics).
    pub(in crate::project_slice) child_order: Vec<(usize, bool)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum ExtrusionRole {
    ExternalPerimeter,
    Perimeter,
    OverhangPerimeter,
    GapFill,
    SolidInfill,
    TopSolidInfill,
    BottomSurface,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct Point3 {
    pub(in crate::project_slice) x: Coord,
    pub(in crate::project_slice) y: Coord,
    pub(in crate::project_slice) z: Coord,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct FittedArc {
    pub(in crate::project_slice) center: (f64, f64),
    pub(in crate::project_slice) radius: f64,
    pub(in crate::project_slice) length: f64,
    pub(in crate::project_slice) clockwise: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct FittedMove {
    pub(in crate::project_slice) start: usize,
    pub(in crate::project_slice) end: usize,
    pub(in crate::project_slice) arc: Option<FittedArc>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct Polyline3 {
    pub(in crate::project_slice) points: Vec<Point3>,
    pub(in crate::project_slice) fitting: Vec<FittedMove>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct ExtrusionPath {
    pub(in crate::project_slice) z_contoured: bool,
    pub(in crate::project_slice) polyline: Polyline3,
    pub(in crate::project_slice) role: ExtrusionRole,
    pub(in crate::project_slice) can_reverse: bool,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
}

impl ExtrusionPath {
    pub(in crate::project_slice) fn reverse(&mut self) {
        self.polyline.reverse();
    }
}
