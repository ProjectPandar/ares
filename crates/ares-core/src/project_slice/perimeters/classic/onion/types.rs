use crate::{
    Project, geometry::ExPolygon, project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::super::PostClassicTopSplitPrintObject;

pub(in crate::project_slice) struct PreparedPostClassicOnion {
    pub(in crate::project_slice) project: Project,
    pub(in crate::project_slice) resolved: Box<BoundedResolvedProjectConfig>,
    pub(in crate::project_slice) config_block: Option<Vec<u8>>,
    pub(in crate::project_slice) scale: crate::geometry::CoordinateScale,
    pub(in crate::project_slice) objects: Vec<PostClassicOnionPrintObject>,
}

pub(in crate::project_slice) struct PostClassicOnionPrintObject {
    pub(in crate::project_slice) predecessor: PostClassicTopSplitPrintObject,
    pub(in crate::project_slice) records: Vec<Option<ClassicOnionRecord>>,
}

#[derive(Debug)]
pub(in crate::project_slice) struct ClassicOnionRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedOnionSurface>,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct RawShellDepth {
    pub(in crate::project_slice) depth: i32,
    pub(in crate::project_slice) normal: Vec<ExPolygon>,
    pub(in crate::project_slice) smaller_width: Vec<ExPolygon>,
}

#[derive(Debug)]
pub(in crate::project_slice) struct PreparedOnionSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) initial_loop_number: i32,
    pub(in crate::project_slice) effective_loop_number: i32,
    pub(in crate::project_slice) shells: Vec<RawShellDepth>,
    pub(in crate::project_slice) last: Vec<ExPolygon>,
    pub(in crate::project_slice) gaps: Vec<ExPolygon>,
}

impl PostClassicOnionPrintObject {
    pub(in crate::project_slice) fn into_parts(
        self,
    ) -> (
        PostClassicTopSplitPrintObject,
        Vec<Option<ClassicOnionRecord>>,
    ) {
        (self.predecessor, self.records)
    }
}
