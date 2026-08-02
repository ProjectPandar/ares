use crate::{
    ProcessBrimType, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::super::entity_collections::ExtrusionEntityCollection;

pub(in crate::project_slice) struct PreparedPostClassicPerimeterAppend {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedPerimeterAppendObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedPerimeterAppendObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedPerimeterAppendRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedPerimeterAppendRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedPerimeterAppendSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedPerimeterAppendSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) inactive: InactivePostCollectionBranches,
    pub(in crate::project_slice) appended: AppendedPerimeterCollections,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct AppendedPerimeterCollections {
    pub(in crate::project_slice) collections: Vec<ExtrusionEntityCollection>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct InactivePostCollectionBranches {
    pub(in crate::project_slice) overhang_reorientation: InactiveOverhangReorientation,
    pub(in crate::project_slice) wall_reordering: InactiveWallReordering,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum InactiveOverhangReorientation {
    Disabled {
        overhang_reverse_internal_only: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) enum InactiveWallReordering {
    InnerOuter {
        outer_brim: InactiveOuterBrimReordering,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) enum InactiveOuterBrimReordering {
    LaterLayer {
        layer_id: usize,
        brim_type: ProcessBrimType,
        brim_width: f64,
    },
    DifferentBrimType {
        brim_type: ProcessBrimType,
        brim_width: f64,
    },
    WidthNotPositive {
        brim_width: f64,
    },
}
