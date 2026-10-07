use crate::{
    Project, ProjectSettings, SliceError,
    options::{
        materialize_project_variants,
        project_config_views::resolve_project_config_views,
        project_fdm_normalization::{ProjectFdmNormalizationKey, normalize_fdm_1, normalize_fdm_2},
    },
};

use super::{
    ValidatedMaterializedProject,
    candidates::{resolve_project_candidates, resolve_project_objects},
    grouping::{GroupedPrintObjects, group_print_object_transforms},
    types::{BoundedProjectUsage, BoundedResolvedProjectConfig, ResolvedProjectObject},
    usage::{ProjectUsageSources, collect_bounded_project_usage},
    validate_materialized_project,
};

pub(super) fn resolve_bounded_project_config(
    project: &Project,
) -> Result<BoundedResolvedProjectConfig, SliceError> {
    let context = crate::options::flush_matrix::PreparationContext::current(
        project.settings(),
        &project.documents().project_settings_raw,
    );
    resolve_bounded_project_config_with_preparation(
        project,
        &context,
        &crate::ConfigOverrides::default(),
    )
}

pub(crate) fn resolve_bounded_project_config_with_preparation(
    project: &Project,
    context: &crate::options::flush_matrix::PreparationContext,
    overrides: &crate::ConfigOverrides,
) -> Result<BoundedResolvedProjectConfig, SliceError> {
    resolve_with_observer(
        project,
        prepared_source(project, context, overrides)?,
        &mut NoopObserver,
    )
}

fn prepared_source(
    project: &Project,
    context: &crate::options::flush_matrix::PreparationContext,
    overrides: &crate::ConfigOverrides,
) -> Result<ProjectSettings, SliceError> {
    let mut source = project.settings().clone();
    crate::options::flush_matrix::prepare(
        &mut source,
        &project.documents().project_settings_raw,
        context,
        overrides,
    )?;
    Ok(source)
}

fn resolve_with_observer(
    project: &Project,
    mut normalized_source: ProjectSettings,
    observer: &mut impl PhaseObserver,
) -> Result<BoundedResolvedProjectConfig, SliceError> {
    normalize_fdm_1(&mut normalized_source);
    observer.normalize_fdm_1(&normalized_source);

    let mut first = normalized_source.clone();
    call_normalize_fdm_2(&mut first, NormalizeFdm2Pass::ColdFirst, 0, 0, observer);
    first = materialize(first, MaterializationPass::First, observer)?;
    let first_validated = validate_materialized_project(&first, project)?;
    let grouped = group_print_object_transforms(project.objects());
    observer.validate_and_group(MaterializationPass::First, &grouped);

    let first_shells =
        resolve_project_objects(&first, first_validated, project.objects(), &grouped)?;
    observer.resolve_object_shells(&first_shells);
    let first_pre_region = collect_usage(
        UsagePass::FirstPreRegion,
        ProjectUsageSources {
            settings: &first,
            objects: project.objects(),
            grouped: &grouped,
            resolved: &first_shells,
        },
        &first,
        observer,
    );

    let print_object_count = grouped.effective_print_object_count;
    call_normalize_fdm_2(
        &mut first,
        NormalizeFdm2Pass::FirstLate,
        print_object_count,
        first_pre_region.supported_used_filaments.len(),
        observer,
    );
    let first_candidates = resolve_candidates(
        CandidatePass::FirstPreliminary,
        &first,
        first_validated,
        (project, &grouped),
        observer,
    )?;
    let second_early = collect_usage(
        UsagePass::SecondEarly,
        ProjectUsageSources {
            settings: &first,
            objects: project.objects(),
            grouped: &grouped,
            resolved: &first_candidates,
        },
        &first,
        observer,
    );

    let mut second = normalized_source;
    call_normalize_fdm_2(
        &mut second,
        NormalizeFdm2Pass::SecondEarly,
        print_object_count,
        second_early.supported_used_filaments.len(),
        observer,
    );
    second = materialize(second, MaterializationPass::Second, observer)?;
    let second_validated = validate_materialized_project(&second, project)?;
    observer.validate_and_group(MaterializationPass::Second, &grouped);

    let second_candidates = resolve_candidates(
        CandidatePass::SecondPreliminary,
        &second,
        second_validated,
        (project, &grouped),
        observer,
    )?;
    let final_pre_normalize = collect_usage(
        UsagePass::FinalPreNormalize,
        ProjectUsageSources {
            settings: &second,
            objects: project.objects(),
            grouped: &grouped,
            resolved: &second_candidates,
        },
        &second,
        observer,
    );
    call_normalize_fdm_2(
        &mut second,
        NormalizeFdm2Pass::SecondLate,
        print_object_count,
        final_pre_normalize.supported_used_filaments.len(),
        observer,
    );
    let usage = collect_usage(
        UsagePass::FinalPostNormalize,
        ProjectUsageSources {
            settings: &second,
            objects: project.objects(),
            grouped: &grouped,
            resolved: &second_candidates,
        },
        &second,
        observer,
    );
    debug_assert_eq!(usage, final_pre_normalize);

    let objects = resolve_candidates(
        CandidatePass::Final,
        &second,
        second_validated,
        (project, &grouped),
        observer,
    )?;
    // Temporary applied-map snapshot until ToolOrdering runs after validation.
    let applied_filament_map = second.project.gcode.filament_map.clone();
    // `ToolOrdering::initialize` re-derives the filament map under
    // `filament_map_mode < Manual` and writes it back into the print
    // config before G-code export (`ToolOrdering.cpp:1288-1303`,
    // `Print::update_filament_maps_to_config`).
    crate::options::filament_map_recommend::apply_recommended_filament_map(&mut second);
    observer.resolve_views(&second);
    let views = resolve_project_config_views(second)?;

    Ok(BoundedResolvedProjectConfig {
        views,
        applied_filament_map,
        logical_filament_count: second_validated.logical_filament_count,
        usage,
        print_object_count,
        objects,
    })
}

fn resolve_candidates(
    pass: CandidatePass,
    settings: &ProjectSettings,
    validated: ValidatedMaterializedProject,
    (project, grouped): (&Project, &GroupedPrintObjects),
    observer: &mut impl PhaseObserver,
) -> Result<Vec<ResolvedProjectObject>, SliceError> {
    let resolved = resolve_project_candidates(settings, validated, project.objects(), grouped)?;
    observer.candidates(pass, &resolved);
    Ok(resolved)
}

fn materialize(
    source: ProjectSettings,
    pass: MaterializationPass,
    observer: &mut impl PhaseObserver,
) -> Result<ProjectSettings, SliceError> {
    let materialized = materialize_project_variants(&source, &source.project.gcode.filament_map)?;
    observer.materialize(pass, &source, &materialized);
    Ok(materialized)
}

fn call_normalize_fdm_2(
    settings: &mut ProjectSettings,
    pass: NormalizeFdm2Pass,
    object_count: usize,
    used_filament_count: usize,
    observer: &mut impl PhaseObserver,
) {
    let keys = normalize_fdm_2(settings, object_count, used_filament_count);
    observer.normalize_fdm_2(pass, (object_count, used_filament_count), &keys, settings);
}

fn collect_usage(
    pass: UsagePass,
    sources: ProjectUsageSources<'_>,
    wipe_settings: &ProjectSettings,
    observer: &mut impl PhaseObserver,
) -> BoundedProjectUsage {
    let resolved = sources.resolved;
    let source_settings = sources.settings;
    let usage = collect_bounded_project_usage(sources, wipe_settings);
    observer.usage(pass, &usage, resolved, (source_settings, wipe_settings));
    usage
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NormalizeFdm2Pass {
    ColdFirst,
    FirstLate,
    SecondEarly,
    SecondLate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MaterializationPass {
    First,
    Second,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidatePass {
    FirstPreliminary,
    SecondPreliminary,
    Final,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UsagePass {
    FirstPreRegion,
    SecondEarly,
    FinalPreNormalize,
    FinalPostNormalize,
}

trait PhaseObserver {
    fn normalize_fdm_1(&mut self, _settings: &ProjectSettings) {}
    fn normalize_fdm_2(
        &mut self,
        _pass: NormalizeFdm2Pass,
        _counts: (usize, usize),
        _changed_keys: &[ProjectFdmNormalizationKey],
        _settings: &ProjectSettings,
    ) {
    }
    fn materialize(
        &mut self,
        _pass: MaterializationPass,
        _source: &ProjectSettings,
        _materialized: &ProjectSettings,
    ) {
    }
    fn validate_and_group(&mut self, _pass: MaterializationPass, _groups: &GroupedPrintObjects) {}
    fn resolve_object_shells(&mut self, _resolved: &[ResolvedProjectObject]) {}
    fn candidates(&mut self, _pass: CandidatePass, _resolved: &[ResolvedProjectObject]) {}
    fn usage(
        &mut self,
        _pass: UsagePass,
        _usage: &BoundedProjectUsage,
        _resolved: &[ResolvedProjectObject],
        _settings: (&ProjectSettings, &ProjectSettings),
    ) {
    }
    fn resolve_views(&mut self, _settings: &ProjectSettings) {}
}

struct NoopObserver;
impl PhaseObserver for NoopObserver {}

#[cfg(test)]
pub(crate) use super::types::{
    CandidateSnapshot, MaterializationSnapshot, NormalizeFdm2Snapshot, PhaseEvent, ResolutionTrace,
    UsageSnapshot,
};

#[cfg(test)]
mod trace;
#[cfg(test)]
pub(crate) use trace::resolve_bounded_project_config_with_trace;
