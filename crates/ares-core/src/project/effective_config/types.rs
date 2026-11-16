use crate::{
    ObjectOptions, OrcaInts, ProjectSettings, RegionOptions,
    options::project_config_views::ProjectConfigViews, project::transform::Transform3d,
};

#[derive(Debug, PartialEq)]
pub(crate) struct BoundedResolvedProjectConfig {
    pub(crate) views: ProjectConfigViews,
    pub(crate) filament_variant_checkpoint: ProjectSettings,
    pub(crate) applied_filament_map: OrcaInts,
    pub(crate) process_nozzle_assignments: Option<Vec<Option<crate::options::ProcessNozzle>>>,
    pub(crate) process_variant_indices: Vec<usize>,
    pub(crate) logical_filament_count: usize,
    pub(crate) usage: BoundedProjectUsage,
    pub(crate) print_object_count: usize,
    pub(crate) objects: Vec<ResolvedProjectObject>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectUsageCoverage {
    TypedConfigSourcesOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundedProjectUsage {
    pub(crate) supported_used_filaments: Vec<usize>,
    pub(crate) coverage: ProjectUsageCoverage,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedPrintObjectConfig {
    pub(crate) transform: Transform3d,
    pub(crate) instance_indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedModelPartCandidate {
    pub(crate) volume_index: usize,
    pub(crate) region: RegionOptions,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedLayerCandidate {
    pub(crate) min_z: f64,
    pub(crate) max_z: f64,
    pub(crate) source_range_index: Option<usize>,
    pub(crate) model_parts: Vec<ResolvedModelPartCandidate>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedProjectObject {
    pub(crate) source_object_index: usize,
    pub(crate) object: ObjectOptions,
    pub(crate) print_objects: Vec<ResolvedPrintObjectConfig>,
    pub(crate) layer_candidates: Vec<ResolvedLayerCandidate>,
    pub(crate) all_regions: Vec<RegionOptions>,
}
