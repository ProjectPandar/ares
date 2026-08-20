use super::super::{chained_loops::ExtrusionLoop, traversal::PreparedPostClassicTraversal};

pub(in crate::project_slice) struct PreparedPostClassicEntityCollections {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedEntityCollectionObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedEntityCollectionObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedEntityCollectionRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedEntityCollectionRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedEntityCollectionSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedEntityCollectionSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) collection: ExtrusionEntityCollection,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct ExtrusionEntityCollection {
    pub(in crate::project_slice) entities: Vec<OrderedExtrusionLoop>,
    pub(in crate::project_slice) source_order: usize,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct OrderedExtrusionLoop {
    pub(in crate::project_slice) extrusion_loop: ExtrusionLoop,
    pub(in crate::project_slice) inset_idx: i32,
}
