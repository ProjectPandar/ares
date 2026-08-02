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
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct RawPathNode {
    pub(in crate::project_slice) paths: Vec<ExtrusionPath>,
    pub(in crate::project_slice) children: Vec<RawPathNode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum ExtrusionRole {
    ExternalPerimeter,
    Perimeter,
    OverhangPerimeter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct Point3 {
    pub(in crate::project_slice) x: Coord,
    pub(in crate::project_slice) y: Coord,
    pub(in crate::project_slice) z: Coord,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct Polyline3 {
    pub(in crate::project_slice) points: Vec<Point3>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct ExtrusionPath {
    pub(in crate::project_slice) polyline: Polyline3,
    pub(in crate::project_slice) role: ExtrusionRole,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
}

impl ExtrusionPath {
    pub(in crate::project_slice) fn reverse(&mut self) {
        self.polyline.points.reverse();
    }
}
