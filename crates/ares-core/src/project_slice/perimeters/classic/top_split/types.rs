use crate::{
    Project, geometry::ExPolygon, project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::super::types::PostClassicPreludePrintObject;

pub(in crate::project_slice) struct PreparedPostClassicTopSplit {
    pub(in crate::project_slice) project: Project,
    pub(in crate::project_slice) resolved: Box<BoundedResolvedProjectConfig>,
    pub(in crate::project_slice) config_block: Option<Vec<u8>>,
    pub(in crate::project_slice) scale: crate::geometry::CoordinateScale,
    pub(in crate::project_slice) objects: Vec<PostClassicTopSplitPrintObject>,
}

pub(in crate::project_slice) struct PostClassicTopSplitPrintObject {
    pub(in crate::project_slice) predecessor: PostClassicPreludePrintObject,
    pub(in crate::project_slice) records: Vec<Option<ClassicTopSplitRecord>>,
}

#[derive(Debug)]
pub(in crate::project_slice) struct ClassicTopSplitRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedTopSplitSurface>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum TopSplitUpperSource {
    WholeLayer,
    SameRegion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum TopSplitOutcome {
    Disabled,
    NoLoops,
    Collapsed,
    OneLoop,
    Bridge,
    NoUpperLayer,
    Applied,
}

#[derive(Debug)]
pub(in crate::project_slice) struct PreparedTopSplitSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) initial_loop_number: i32,
    pub(in crate::project_slice) effective_loop_number: i32,
    pub(in crate::project_slice) normal_first_offset: Vec<ExPolygon>,
    pub(in crate::project_slice) smaller_first_offset: Vec<ExPolygon>,
    /// `thin_walls` — `i == 0` medial-axis extractions for `detect_thin_wall`
    /// (PerimeterGenerator.cpp:1266). Upstream declares `thin_walls` inside the
    /// per-surface loop (:1222,1243) and emits them in that surface's
    /// `traverse_loops` (:1450), so this stays a PER-SURFACE field carried into
    /// that surface's traversal collection — not record-level. Empty unless
    /// `detect_thin_wall`.
    pub(in crate::project_slice) thin_walls: Vec<crate::geometry::ThickPolyline>,
    pub(in crate::project_slice) remaining: Vec<ExPolygon>,
    pub(in crate::project_slice) top_fills: Vec<ExPolygon>,
    pub(in crate::project_slice) fill_clip: Vec<ExPolygon>,
    pub(in crate::project_slice) outcome: TopSplitOutcome,
    pub(in crate::project_slice) upper_source: TopSplitUpperSource,
}

impl PostClassicTopSplitPrintObject {
    pub(in crate::project_slice) fn into_parts(
        self,
    ) -> (
        PostClassicPreludePrintObject,
        Vec<Option<ClassicTopSplitRecord>>,
    ) {
        (self.predecessor, self.records)
    }
}
