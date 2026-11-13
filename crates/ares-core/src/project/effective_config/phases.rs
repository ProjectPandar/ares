use crate::{
    Project, ProjectBedType, ProjectSettings, SliceError,
    options::{
        MaterializedProjectVariants, materialize_project_variants,
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
    if source.project.print.curr_bed_type == ProjectBedType::DefaultPlate {
        // Preset.cpp:1034–1048: without a preset bundle, the model identifier is empty.
        source.project.print.curr_bed_type = source.printer.remaining.default_bed_type.resolve("");
    }
    Ok(source)
}

fn resolve_with_observer(
    project: &Project,
    mut normalized_source: ProjectSettings,
    observer: &mut impl PhaseObserver,
) -> Result<BoundedResolvedProjectConfig, SliceError> {
    normalize_fdm_1(&mut normalized_source);
    observer.normalize_fdm_1(&normalized_source);
    super::scarf_joint_seam::apply(&mut normalized_source, project.objects());

    let mut first = normalized_source;
    call_normalize_fdm_2(&mut first, NormalizeFdm2Pass::ColdFirst, 0, 0, observer);
    // The raw-map entry guard must run before materialization rewrites the
    // slots (grow + single-extruder reset, `OrcaSlicer.cpp:6027-6034`).
    super::cardinality::validate_source_filament_map_entries(&first)?;
    let MaterializedProjectVariants {
        full: materialized,
        checkpoint: filament_variant_checkpoint,
        process_variant_indices,
    } = materialize(first, MaterializationPass::First, observer)?;
    first = materialized;
    let first_validated = validate_materialized_project(&first, project)?;
    let grouped = group_print_object_transforms(project.objects());
    observer.validate_and_group(MaterializationPass::First, &grouped);

    let first_shells = resolve_project_objects(
        &first,
        first_validated,
        project.objects(),
        &grouped,
        &process_variant_indices,
    )?;
    observer.resolve_object_shells(&first_shells);
    let first_pre_region = collect_usage(
        UsagePass::FirstPreRegion,
        ProjectUsageSources {
            settings: &first,
            objects: project.objects(),
            grouped: &grouped,
            resolved: &first_shells,
            custom_tool_changes: &[],
        },
        &first,
        observer,
    );

    let print_object_count = grouped.effective_print_object_count;
    let mut unnormalized = first.clone();
    let first_changed_keys = call_normalize_fdm_2(
        &mut first,
        NormalizeFdm2Pass::FirstLate,
        print_object_count,
        first_pre_region.supported_used_filaments.len(),
        observer,
    );
    // CLI calls Print::apply once (OrcaSlicer.cpp:6040); its late
    // normalize_fdm_2 precedes region construction (PrintApply.cpp:1620-1621).
    let objects = resolve_candidates(
        CandidatePass::Final,
        &first,
        first_validated,
        (project, &grouped, &process_variant_indices),
        observer,
    )?;
    let custom_tool_changes = project
        .curr_plate_custom_gcodes()
        .gcodes
        .into_iter()
        .filter(|item| item.kind == crate::gcode_processor::CustomGCodeType::ToolChange)
        .map(|item| item.extruder)
        .collect::<Vec<_>>();
    let post_region_usage = collect_usage(
        UsagePass::PostRegion,
        ProjectUsageSources {
            settings: &first,
            objects: project.objects(),
            grouped: &grouped,
            resolved: &objects,
            custom_tool_changes: &custom_tool_changes,
        },
        &first,
        observer,
    );
    // PrintApply.cpp:2208-2216 rechecks the retained configuration, then
    // applies only keys that differ from the configuration already applied.
    let mut changed_keys = call_normalize_fdm_2(
        &mut unnormalized,
        NormalizeFdm2Pass::PostRegion,
        print_object_count,
        post_region_usage.supported_used_filaments.len(),
        observer,
    );
    changed_keys.extend(first_changed_keys);
    changed_keys.retain(|key| match key {
        ProjectFdmNormalizationKey::EnablePrimeTower => {
            first.process.print.enable_prime_tower != unnormalized.process.print.enable_prime_tower
        }
        ProjectFdmNormalizationKey::IndependentSupportLayerHeight => {
            first.process.print.independent_support_layer_height
                != unnormalized.process.print.independent_support_layer_height
        }
    });
    for key in changed_keys {
        match key {
            ProjectFdmNormalizationKey::EnablePrimeTower => {
                first.process.print.enable_prime_tower.0 =
                    unnormalized.process.print.enable_prime_tower.0;
            }
            ProjectFdmNormalizationKey::IndependentSupportLayerHeight => {
                first.process.print.independent_support_layer_height.0 = unnormalized
                    .process
                    .print
                    .independent_support_layer_height
                    .0;
            }
        }
    }
    let usage = collect_usage(
        UsagePass::FinalPostNormalize,
        ProjectUsageSources {
            settings: &first,
            objects: project.objects(),
            grouped: &grouped,
            resolved: &objects,
            custom_tool_changes: &custom_tool_changes,
        },
        &first,
        observer,
    );
    // Temporary applied-map snapshot until ToolOrdering runs after validation.
    let applied_filament_map = first.project.gcode.filament_map.clone();
    crate::options::filament_map_recommend::prepare_multi_extruder_cli_defaults(&mut first);
    // ToolOrdering::initialize re-derives the map before G-code export
    // (ToolOrdering.cpp:1288-1303; Print::update_filament_maps_to_config).
    crate::options::filament_map_recommend::apply_recommended_filament_map(&mut first);
    observer.resolve_views(&first);
    let views = resolve_project_config_views(first)?;

    Ok(BoundedResolvedProjectConfig {
        views,
        filament_variant_checkpoint,
        applied_filament_map,
        process_variant_indices,
        logical_filament_count: first_validated.logical_filament_count,
        usage,
        print_object_count,
        objects,
    })
}

fn resolve_candidates(
    pass: CandidatePass,
    settings: &ProjectSettings,
    validated: ValidatedMaterializedProject,
    (project, grouped, process_variant_indices): (&Project, &GroupedPrintObjects, &[usize]),
    observer: &mut impl PhaseObserver,
) -> Result<Vec<ResolvedProjectObject>, SliceError> {
    let resolved = resolve_project_candidates(
        settings,
        validated,
        project.objects(),
        grouped,
        process_variant_indices,
    )?;
    observer.candidates(pass, &resolved);
    Ok(resolved)
}

fn materialize(
    source: ProjectSettings,
    pass: MaterializationPass,
    observer: &mut impl PhaseObserver,
) -> Result<MaterializedProjectVariants, SliceError> {
    let materialized = materialize_project_variants(&source, &source.project.gcode.filament_map)?;
    observer.materialize(pass, &source, &materialized.full);
    Ok(materialized)
}

fn call_normalize_fdm_2(
    settings: &mut ProjectSettings,
    pass: NormalizeFdm2Pass,
    object_count: usize,
    used_filament_count: usize,
    observer: &mut impl PhaseObserver,
) -> Vec<ProjectFdmNormalizationKey> {
    let keys = normalize_fdm_2(settings, object_count, used_filament_count);
    observer.normalize_fdm_2(pass, (object_count, used_filament_count), &keys, settings);
    keys
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
    PostRegion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MaterializationPass {
    First,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidatePass {
    Final,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UsagePass {
    FirstPreRegion,
    PostRegion,
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
