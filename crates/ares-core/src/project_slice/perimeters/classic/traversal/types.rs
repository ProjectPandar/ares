use crate::project::effective_config::types::BoundedResolvedProjectConfig;
use crate::project_slice::perimeters::types::Flow;
use crate::{Project, geometry::CoordinateScale};

use super::super::hierarchy::{PerimeterGeneratorLoop, PostClassicHierarchyPrintObject};

pub(in crate::project_slice) struct PreparedPostClassicTraversal {
    pub(in crate::project_slice) project: Project,
    pub(in crate::project_slice) resolved: Box<BoundedResolvedProjectConfig>,
    pub(in crate::project_slice) config_block: Option<Vec<u8>>,
    pub(in crate::project_slice) scale: CoordinateScale,
    pub(in crate::project_slice) objects: Vec<PostClassicTraversalPrintObject>,
}

pub(in crate::project_slice) struct PostClassicTraversalPrintObject {
    pub(in crate::project_slice) predecessor: PostClassicHierarchyPrintObject,
    pub(in crate::project_slice) records: Vec<Option<ClassicTraversalRecord>>,
}

#[derive(Debug)]
pub(in crate::project_slice) struct ClassicTraversalRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedTraversalSurface>,
    pub(in crate::project_slice) layer_height: f64,
    pub(in crate::project_slice) overhang_flow: Flow,
    pub(in crate::project_slice) branch: PendingPathBranch,
    pub(in crate::project_slice) overhang_reverse: InactiveOverhangReverse,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedTraversalSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) roots: Vec<TraversalSeed>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum PendingExtrusionRole {
    ExternalPerimeter,
    Perimeter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum PendingLoopRole {
    Internal,
    Default,
    Hole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum LowerFlowRoute {
    SmallerExternal,
    External,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum PendingPathBranch {
    OverhangClipping {
        detect_overhang_wall: bool,
        layer_id: usize,
        raft_layers: i32,
    },
    OrdinaryUnsplit {
        detect_overhang_wall: bool,
        layer_id: usize,
        raft_layers: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct InactiveOverhangReverse {
    pub(in crate::project_slice) configured: bool,
    pub(in crate::project_slice) odd_layer: bool,
    pub(in crate::project_slice) active: bool,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct TraversalSeed {
    pub(in crate::project_slice) polygon: crate::geometry::Polygon,
    pub(in crate::project_slice) depth: u16,
    pub(in crate::project_slice) is_contour: bool,
    pub(in crate::project_slice) is_smaller_width_perimeter: bool,
    pub(in crate::project_slice) extrusion_role: PendingExtrusionRole,
    pub(in crate::project_slice) loop_role: PendingLoopRole,
    pub(in crate::project_slice) route: LowerFlowRoute,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) children: Vec<TraversalSeed>,
}

impl PendingPathBranch {
    pub(in crate::project_slice) fn from_operands(
        detect_overhang_wall: bool,
        layer_id: usize,
        raft_layers: i32,
    ) -> Self {
        if detect_overhang_wall
            && i32::try_from(layer_id).expect("validated layer_id fits the source int")
                > raft_layers
        {
            Self::OverhangClipping {
                detect_overhang_wall,
                layer_id,
                raft_layers,
            }
        } else {
            Self::OrdinaryUnsplit {
                detect_overhang_wall,
                layer_id,
                raft_layers,
            }
        }
    }
}

impl PostClassicTraversalPrintObject {
    pub(in crate::project_slice) fn into_parts(
        self,
    ) -> (
        PostClassicHierarchyPrintObject,
        Vec<Option<ClassicTraversalRecord>>,
    ) {
        (self.predecessor, self.records)
    }
}

#[derive(Clone, Copy)]
pub(super) struct RouteFlows {
    pub(super) perimeter: Flow,
    pub(super) external: Flow,
    pub(super) smaller_external: Flow,
}

pub(super) struct SeedFrame<'a> {
    pub(super) source: &'a PerimeterGeneratorLoop,
    pub(super) next_child: usize,
    pub(super) children: Vec<TraversalSeed>,
}
