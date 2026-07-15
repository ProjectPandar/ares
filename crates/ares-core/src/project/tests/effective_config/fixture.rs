use crate::{
    OrcaBool, OrcaFloat, OrcaFloats, ZHopType,
    options::project_config_views::ProjectConfigViews,
    project::effective_config::{
        phases::{UsagePass, resolve_bounded_project_config_with_trace},
        resolve_bounded_project_config,
        types::{BoundedResolvedProjectConfig, ProjectUsageCoverage},
    },
};

use super::{load_project, support::ProjectParts};

#[test]
fn committed_project_resolves_one_bounded_final_configuration() {
    let project = load_project(ProjectParts::fixture().bytes()).unwrap();
    let raw = project.settings().clone();
    let raw_retraction_stride = raw.project.gcode.retraction_length.clone();
    let raw_filament_retraction_stride = raw
        .filament
        .retract_overrides
        .filament_retraction_length
        .clone();

    let resolved = resolve_bounded_project_config(&project).unwrap();
    let (_, trace) = resolve_bounded_project_config_with_trace(&project).unwrap();

    assert_fixture_shape(&resolved);
    assert_fixture_views(&resolved.views);
    assert_eq!(project.settings(), &raw);
    assert_eq!(
        project.settings().project.gcode.retraction_length,
        raw_retraction_stride
    );
    assert_eq!(
        project
            .settings()
            .filament
            .retract_overrides
            .filament_retraction_length,
        raw_filament_retraction_stride
    );
    assert_eq!(raw_retraction_stride.0.len(), 4);
    assert_eq!(raw_filament_retraction_stride.len(), 8);
    let second_early = trace
        .usages
        .iter()
        .find(|usage| usage.pass == UsagePass::SecondEarly)
        .unwrap();
    assert!(!trace.normalize_fdm_2[1].enable_prime_tower);
    assert!(!second_early.source_enable_prime_tower);
    assert!(!second_early.wipe_enable_prime_tower);
}

fn assert_fixture_shape(resolved: &BoundedResolvedProjectConfig) {
    assert_eq!(resolved.logical_filament_count, 2);
    assert_eq!(resolved.print_object_count, 1);
    assert_eq!(resolved.objects.len(), 1);
    assert_eq!(resolved.objects[0].print_objects.len(), 1);
    assert_eq!(resolved.objects[0].layer_candidates.len(), 1);

    let candidate = &resolved.objects[0].layer_candidates[0];
    assert_eq!((candidate.min_z, candidate.max_z), (0.0, f64::MAX));
    assert_eq!(candidate.source_range_index, None);
    assert_eq!(candidate.model_parts.len(), 1);
    assert_eq!(candidate.model_parts[0].volume_index, 0);
    assert_eq!(resolved.usage.supported_used_filaments, [0]);
    assert_eq!(
        resolved.usage.coverage,
        ProjectUsageCoverage::TypedConfigSourcesOnly
    );
}

fn assert_fixture_views(views: &ProjectConfigViews) {
    assert_eq!(views.full.process.print.enable_prime_tower, OrcaBool(false));
    assert_eq!(
        views.full.process.print.independent_support_layer_height,
        OrcaBool(true)
    );
    assert_eq!(views.full.process.print.resolution, OrcaFloat(0.012));
    assert_eq!(
        views.full.project.gcode.retraction_length,
        OrcaFloats(vec![OrcaFloat(0.8), OrcaFloat(2.0)])
    );
    assert_eq!(
        views.runtime.project.gcode.retraction_length,
        OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)])
    );
    assert_eq!(
        views.runtime_gcode.retraction_length,
        views.runtime.project.gcode.retraction_length
    );
    assert_eq!(
        views.full.project.gcode.deretraction_speed,
        OrcaFloats(vec![OrcaFloat(30.0), OrcaFloat(20.0)])
    );
    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        OrcaFloats(vec![OrcaFloat(30.0), OrcaFloat(30.0)])
    );
    assert_eq!(
        views.full.printer.gcode.retraction_distances_when_cut,
        OrcaFloats(vec![OrcaFloat(18.0), OrcaFloat(18.0)])
    );
    assert_eq!(
        views.runtime.printer.gcode.retraction_distances_when_cut,
        OrcaFloats(vec![OrcaFloat(10.0), OrcaFloat(10.0)])
    );
    assert_eq!(
        views.full.project.gcode.retraction_speed,
        OrcaFloats(vec![OrcaFloat(30.0), OrcaFloat(20.0)])
    );
    assert_eq!(
        views.runtime.project.gcode.retraction_speed,
        OrcaFloats(vec![OrcaFloat(30.0), OrcaFloat(30.0)])
    );
    assert_eq!(
        views.full.project.print.wipe_distance,
        OrcaFloats(vec![OrcaFloat(2.0), OrcaFloat(2.0)])
    );
    assert_eq!(
        views.runtime.project.print.wipe_distance,
        OrcaFloats(vec![OrcaFloat(1.0), OrcaFloat(1.0)])
    );
    assert_eq!(
        views.full.printer.gcode.z_hop_types.0,
        [ZHopType::Auto, ZHopType::Auto]
    );
    assert_eq!(
        views.runtime.printer.gcode.z_hop_types.0,
        [ZHopType::Spiral, ZHopType::Spiral]
    );
    assert_eq!(
        views.runtime_gcode.retraction_speed,
        views.runtime.project.gcode.retraction_speed
    );
}
