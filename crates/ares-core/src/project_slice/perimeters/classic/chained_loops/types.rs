use super::super::{materialize::ExtrusionPath, traversal::PreparedPostClassicTraversal};

pub(in crate::project_slice) struct PreparedPostClassicChainedLoops {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedChainedLoopObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedChainedLoopObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedChainedLoopRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedChainedLoopRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedChainedLoopSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedChainedLoopSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) roots: Vec<ChainedLoopNode>,
    /// Root-level chain from materialization (`PerimeterGenerator.cpp:238`),
    /// passed through for the emission traversal to consume without
    /// recomputation.
    pub(in crate::project_slice) root_order: Vec<(usize, bool)>,
    /// Thin-wall entities from materialization (`PerimeterGenerator.cpp:231`).
    pub(in crate::project_slice) thin_entities:
        Vec<crate::project_slice::perimeters::classic::entity_collections::ExtrusionEntity>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct ChainedLoopNode {
    // None preserves O5 alignment for source line 208 `continue`; it is not an entity.
    pub(in crate::project_slice) extrusion_loop: Option<ExtrusionLoop>,
    pub(in crate::project_slice) children: Vec<ChainedLoopNode>,
    /// Chain over this node's children level, computed once at materialization
    /// (`PerimeterGenerator.cpp:238`) and consumed by the emission traversal.
    pub(in crate::project_slice) child_order: Vec<(usize, bool)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum ExtrusionLoopRole {
    Internal,
    Default,
    Hole,
}

/// Paths and loop role from `libslic3r/ExtrusionEntity.hpp::ExtrusionLoop`.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct ExtrusionLoop {
    pub(in crate::project_slice) paths: Vec<ExtrusionPath>,
    pub(in crate::project_slice) role: ExtrusionLoopRole,
    pub(in crate::project_slice) print_after_infill: bool,
}

impl ExtrusionLoop {
    /// `ExtrusionLoop::reverse` (`ExtrusionEntity.cpp`): reverse the path
    /// order, then reverse every path.
    pub(in crate::project_slice) fn reverse(&mut self) {
        self.paths.reverse();
        for path in &mut self.paths {
            path.reverse();
        }
    }
}
