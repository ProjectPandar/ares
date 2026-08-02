use crate::{
    Project,
    geometry::{CoordinateScale, Polygon},
    project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::super::onion::PostClassicOnionPrintObject;

pub(in crate::project_slice) struct PreparedPostClassicHierarchy {
    pub(in crate::project_slice) project: Project,
    pub(in crate::project_slice) resolved: Box<BoundedResolvedProjectConfig>,
    pub(in crate::project_slice) config_block: Option<Vec<u8>>,
    pub(in crate::project_slice) scale: CoordinateScale,
    pub(in crate::project_slice) objects: Vec<PostClassicHierarchyPrintObject>,
}

pub(in crate::project_slice) struct PostClassicHierarchyPrintObject {
    pub(in crate::project_slice) predecessor: PostClassicOnionPrintObject,
    pub(in crate::project_slice) records: Vec<Option<ClassicHierarchyRecord>>,
}

pub(in crate::project_slice) struct ClassicHierarchyRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedHierarchySurface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct PerimeterGeneratorLoop {
    pub(in crate::project_slice) polygon: Polygon,
    pub(in crate::project_slice) is_contour: bool,
    pub(in crate::project_slice) is_smaller_width_perimeter: bool,
    pub(in crate::project_slice) depth: u16,
    pub(in crate::project_slice) children: Vec<PerimeterGeneratorLoop>,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct PreparedHierarchySurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) roots: Vec<PerimeterGeneratorLoop>,
    pub(in crate::project_slice) remaining_contours: Vec<Vec<PerimeterGeneratorLoop>>,
    pub(in crate::project_slice) remaining_holes: Vec<Vec<PerimeterGeneratorLoop>>,
}

impl PostClassicHierarchyPrintObject {
    pub(in crate::project_slice) fn into_parts(
        self,
    ) -> (
        PostClassicOnionPrintObject,
        Vec<Option<ClassicHierarchyRecord>>,
    ) {
        (self.predecessor, self.records)
    }
}

pub(super) struct LoopBuckets {
    pub(super) contours: Vec<Vec<PerimeterGeneratorLoop>>,
    pub(super) holes: Vec<Vec<PerimeterGeneratorLoop>>,
}
