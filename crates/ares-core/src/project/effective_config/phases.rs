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
    resolve_with_observer(project, &mut NoopObserver)
}

fn resolve_with_observer(
    project: &Project,
    observer: &mut impl PhaseObserver,
) -> Result<BoundedResolvedProjectConfig, SliceError> {
    let mut normalized_source = project.settings().clone();
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
    observer.resolve_views(&second);
    let views = resolve_project_config_views(second)?;

    Ok(BoundedResolvedProjectConfig {
        views,
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
impl PhaseObserver for ResolutionTrace {
    fn normalize_fdm_1(&mut self, _settings: &ProjectSettings) {
        self.events.push(PhaseEvent::NormalizeFdm1);
    }

    fn normalize_fdm_2(
        &mut self,
        pass: NormalizeFdm2Pass,
        (object_count, used_filament_count): (usize, usize),
        changed_keys: &[ProjectFdmNormalizationKey],
        _settings: &ProjectSettings,
    ) {
        self.events.push(PhaseEvent::NormalizeFdm2(pass));
        self.normalize_fdm_2.push(NormalizeFdm2Snapshot {
            pass,
            object_count,
            used_filament_count,
            changed_keys: changed_keys.to_vec(),
            enable_prime_tower: _settings.process.print.enable_prime_tower.0,
        });
    }

    fn materialize(
        &mut self,
        pass: MaterializationPass,
        source: &ProjectSettings,
        materialized: &ProjectSettings,
    ) {
        self.events.push(PhaseEvent::Materialize(pass));
        self.materializations.push(MaterializationSnapshot {
            pass,
            source_retraction_length: source.project.gcode.retraction_length.0.len(),
            materialized_retraction_length: materialized.project.gcode.retraction_length.0.len(),
            source_filament_retraction_length: source
                .filament
                .retract_overrides
                .filament_retraction_length
                .len(),
            materialized_filament_retraction_length: materialized
                .filament
                .retract_overrides
                .filament_retraction_length
                .len(),
            source_retracts_all_false: all_retracts_false(source),
            materialized_retracts_all_false: all_retracts_false(materialized),
            source_filament_retracts_all_false: all_filament_retracts_false(source),
            materialized_filament_retracts_all_false: all_filament_retracts_false(materialized),
            source_enable_prime_tower: source.process.print.enable_prime_tower.0,
            materialized_enable_prime_tower: materialized.process.print.enable_prime_tower.0,
        });
    }

    fn validate_and_group(&mut self, pass: MaterializationPass, _groups: &GroupedPrintObjects) {
        self.events.push(PhaseEvent::ValidateAndGroup(pass));
    }

    fn resolve_object_shells(&mut self, _resolved: &[ResolvedProjectObject]) {
        self.events.push(PhaseEvent::ResolveObjectShells);
    }

    fn candidates(&mut self, pass: CandidatePass, resolved: &[ResolvedProjectObject]) {
        self.events.push(PhaseEvent::Candidates(pass));
        self.candidates.push(CandidateSnapshot {
            pass,
            outer_wall_selectors: resolved
                .iter()
                .flat_map(|object| &object.layer_candidates)
                .flat_map(|candidate| &candidate.model_parts)
                .map(|part| part.region.outer_wall_filament_id.0)
                .collect(),
        });
    }

    fn usage(
        &mut self,
        pass: UsagePass,
        usage: &BoundedProjectUsage,
        resolved: &[ResolvedProjectObject],
        (source_settings, wipe_settings): (&ProjectSettings, &ProjectSettings),
    ) {
        self.events.push(PhaseEvent::Usage(pass));
        self.usages.push(UsageSnapshot {
            pass,
            supported_used_filaments: usage.supported_used_filaments.clone(),
            model_part_count: resolved
                .iter()
                .flat_map(|object| &object.layer_candidates)
                .map(|candidate| candidate.model_parts.len())
                .sum(),
            support_object_count: resolved
                .iter()
                .filter(|object| object.object.enable_support.0)
                .count(),
            source_enable_prime_tower: source_settings.process.print.enable_prime_tower.0,
            wipe_enable_prime_tower: wipe_settings.process.print.enable_prime_tower.0,
            wipe_selector: wipe_settings.process.print.wipe_tower_filament.0,
        });
    }

    fn resolve_views(&mut self, _settings: &ProjectSettings) {
        self.events.push(PhaseEvent::ResolveViews);
        self.view_resolutions += 1;
    }
}

#[cfg(test)]
fn all_retracts_false(settings: &ProjectSettings) -> bool {
    let retracts = &settings.project.print.retract_when_changing_layer.0;
    retracts.iter().all(|value| !value.0)
}

#[cfg(test)]
fn all_filament_retracts_false(settings: &ProjectSettings) -> bool {
    settings
        .filament
        .retract_overrides
        .filament_retract_when_changing_layer
        .iter()
        .all(|value| matches!(value, crate::Nullable::Value(crate::OrcaBool(false))))
}

#[cfg(test)]
pub(crate) fn resolve_bounded_project_config_with_trace(
    project: &Project,
) -> Result<(BoundedResolvedProjectConfig, ResolutionTrace), SliceError> {
    let mut trace = ResolutionTrace::default();
    let resolved = resolve_with_observer(project, &mut trace)?;
    Ok((resolved, trace))
}
