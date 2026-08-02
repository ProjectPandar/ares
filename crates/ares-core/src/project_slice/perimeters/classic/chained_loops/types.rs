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
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct ChainedLoopNode {
    // None preserves O5 alignment for source line 208 `continue`; it is not an entity.
    pub(in crate::project_slice) extrusion_loop: Option<ExtrusionLoop>,
    pub(in crate::project_slice) children: Vec<ChainedLoopNode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum ExtrusionLoopRole {
    Internal,
    Default,
    Hole,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct ExtrusionLoop {
    pub(in crate::project_slice) paths: Vec<ExtrusionPath>,
    pub(in crate::project_slice) role: ExtrusionLoopRole,
}
