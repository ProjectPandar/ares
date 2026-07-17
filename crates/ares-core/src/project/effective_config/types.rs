use crate::{
    ObjectOptions, RegionOptions, options::project_config_views::ProjectConfigViews,
    project::transform::Transform3d,
};

#[derive(Debug, PartialEq)]
pub(crate) struct BoundedResolvedProjectConfig {
    pub(crate) views: ProjectConfigViews,
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
}

#[cfg(test)]
use super::phases::{CandidatePass, MaterializationPass, NormalizeFdm2Pass, UsagePass};

#[cfg(test)]
use crate::options::project_fdm_normalization::ProjectFdmNormalizationKey;

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PhaseEvent {
    NormalizeFdm1,
    NormalizeFdm2(NormalizeFdm2Pass),
    Materialize(MaterializationPass),
    ValidateAndGroup(MaterializationPass),
    ResolveObjectShells,
    Usage(UsagePass),
    Candidates(CandidatePass),
    ResolveViews,
}

#[cfg(test)]
pub(crate) struct NormalizeFdm2Snapshot {
    pub(crate) pass: NormalizeFdm2Pass,
    pub(crate) object_count: usize,
    pub(crate) used_filament_count: usize,
    pub(crate) changed_keys: Vec<ProjectFdmNormalizationKey>,
    pub(crate) enable_prime_tower: bool,
}

#[cfg(test)]
pub(crate) struct MaterializationSnapshot {
    pub(crate) pass: MaterializationPass,
    pub(crate) source_retraction_length: usize,
    pub(crate) materialized_retraction_length: usize,
    pub(crate) source_filament_retraction_length: usize,
    pub(crate) materialized_filament_retraction_length: usize,
    pub(crate) source_retracts_all_false: bool,
    pub(crate) materialized_retracts_all_false: bool,
    pub(crate) source_filament_retracts_all_false: bool,
    pub(crate) materialized_filament_retracts_all_false: bool,
    pub(crate) source_enable_prime_tower: bool,
    pub(crate) materialized_enable_prime_tower: bool,
}

#[cfg(test)]
pub(crate) struct UsageSnapshot {
    pub(crate) pass: UsagePass,
    pub(crate) supported_used_filaments: Vec<usize>,
    pub(crate) model_part_count: usize,
    pub(crate) support_object_count: usize,
    pub(crate) source_enable_prime_tower: bool,
    pub(crate) wipe_enable_prime_tower: bool,
    pub(crate) wipe_selector: i32,
}

#[cfg(test)]
pub(crate) struct CandidateSnapshot {
    pub(crate) pass: CandidatePass,
    pub(crate) outer_wall_selectors: Vec<i32>,
}

#[cfg(test)]
#[derive(Default)]
pub(crate) struct ResolutionTrace {
    pub(crate) events: Vec<PhaseEvent>,
    pub(crate) normalize_fdm_2: Vec<NormalizeFdm2Snapshot>,
    pub(crate) materializations: Vec<MaterializationSnapshot>,
    pub(crate) usages: Vec<UsageSnapshot>,
    pub(crate) candidates: Vec<CandidateSnapshot>,
    pub(crate) view_resolutions: usize,
}
