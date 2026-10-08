use crate::{
    OrcaBool,
    options::project_fdm_normalization::ProjectFdmNormalizationKey,
    project::effective_config::phases::{
        CandidatePass, MaterializationPass, NormalizeFdm2Pass, PhaseEvent, UsagePass,
        resolve_bounded_project_config_with_trace,
    },
};

use super::{load_project, support::ProjectParts};

#[test]
fn cli_single_apply_preserves_sources_and_builds_regions_after_late_normalization() {
    let mut parts = ProjectParts::fixture();
    for (from, to) in [
        (r#""spiral_mode": "0""#, r#""spiral_mode": "1""#),
        (r#""timelapse_type": "0""#, r#""timelapse_type": "1""#),
        (
            r#""wipe_tower_filament": "0""#,
            r#""wipe_tower_filament": "1""#,
        ),
    ] {
        parts.replace("Metadata/project_settings.config", from, to);
    }
    parts.replace("Metadata/model_settings.config", r#"<object id="2">"#,
        r#"<object id="2"><metadata key="enable_support" value="1"/><metadata key="support_filament" value="0"/><metadata key="support_interface_filament" value="0"/>"#);
    parts.replace(
        "Metadata/model_settings.config",
        r#"<part id="1" subtype="normal_part">"#,
        r#"<part id="1" subtype="normal_part"><metadata key="outer_wall_filament_id" value="2"/>"#,
    );
    let project = load_project(parts.bytes()).unwrap();
    let raw = project.settings().clone();
    let (resolved, trace) = resolve_bounded_project_config_with_trace(&project).unwrap();

    assert_eq!(trace.events, expected_events());
    assert_eq!(
        trace
            .normalize_fdm_2
            .iter()
            .map(|call| (call.pass, call.object_count, call.used_filament_count))
            .collect::<Vec<_>>(),
        [
            (NormalizeFdm2Pass::ColdFirst, 0, 0),
            (NormalizeFdm2Pass::FirstLate, 1, 1)
        ]
    );
    assert!(trace.normalize_fdm_2[0].changed_keys.is_empty());
    assert_eq!(
        trace.normalize_fdm_2[1].changed_keys,
        [ProjectFdmNormalizationKey::IndependentSupportLayerHeight]
    );
    assert_eq!(trace.materializations.len(), 1);
    let materialized = &trace.materializations[0];
    assert_eq!(materialized.pass, MaterializationPass::First);
    assert_eq!(
        (
            materialized.source_retraction_length,
            materialized.materialized_retraction_length,
            materialized.source_filament_retraction_length,
            materialized.materialized_filament_retraction_length
        ),
        (4, 2, 8, 2)
    );
    assert!(
        materialized.source_retracts_all_false
            && materialized.materialized_retracts_all_false
            && materialized.source_filament_retracts_all_false
            && materialized.materialized_filament_retracts_all_false
    );
    assert_eq!(trace.usages.len(), 2);
    assert_eq!(trace.usages[0].pass, UsagePass::FirstPreRegion);
    assert_eq!(trace.usages[0].model_part_count, 0);
    assert_eq!(trace.usages[0].support_object_count, 1);
    assert!(trace.usages[0].source_enable_prime_tower && trace.usages[0].wipe_enable_prime_tower);
    assert_eq!(trace.usages[0].wipe_selector, 1);
    assert_eq!(trace.usages[0].supported_used_filaments, [0]);
    assert_eq!(trace.usages[1].pass, UsagePass::FinalPostNormalize);
    assert_eq!(trace.usages[1].model_part_count, 1);
    assert_eq!(trace.usages[1].supported_used_filaments, [0, 1]);
    assert!(trace.usages[1].wipe_enable_prime_tower);
    assert_eq!(trace.candidates.len(), 1);
    assert_eq!(trace.candidates[0].pass, CandidatePass::Final);
    assert_eq!(trace.view_resolutions, 1);
    assert_eq!(project.settings(), &raw);
    assert_eq!(resolved.usage.supported_used_filaments, [0, 1]);
}

#[test]
fn by_object_zero_pre_region_usage_skips_late_normalization() {
    let mut parts = two_z_group_fixture();
    parts.replace(
        "Metadata/project_settings.config",
        r#""print_sequence": "by layer""#,
        r#""print_sequence": "by object""#,
    );
    parts.replace(
        "Metadata/project_settings.config",
        r#""wipe_tower_filament": "0""#,
        r#""wipe_tower_filament": "1""#,
    );
    set_zero_raw_and_two_feature_filaments(&mut parts);
    let project = load_project(parts.bytes()).unwrap();
    let (resolved, trace) = resolve_bounded_project_config_with_trace(&project).unwrap();
    assert_eq!(trace.events, expected_events());
    assert_eq!(
        trace
            .normalize_fdm_2
            .iter()
            .map(|call| (call.pass, call.object_count, call.used_filament_count))
            .collect::<Vec<_>>(),
        [
            (NormalizeFdm2Pass::ColdFirst, 0, 0),
            (NormalizeFdm2Pass::FirstLate, 2, 0)
        ]
    );
    // PrintConfig.cpp:8692 guards both predicates with used_filaments > 0.
    assert!(trace.normalize_fdm_2[1].changed_keys.is_empty());
    assert!(trace.usages[0].supported_used_filaments.is_empty());
    assert!(trace.usages[1].source_enable_prime_tower && trace.usages[1].wipe_enable_prime_tower);
    assert_eq!(resolved.usage.supported_used_filaments, [0, 1]);
    assert_eq!(
        resolved.views.full.process.print.enable_prime_tower,
        OrcaBool(true)
    );
}

#[test]
fn reverse_two_z_groups_never_count_the_nonrepresentative_selector() {
    let mut parts = two_z_group_fixture();
    set_zero_raw_selectors(&mut parts);
    parts.insert_text("Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1"><range min_z="100" max_z="192"><option opt_key="outer_wall_filament_id">2</option></range></object></objects>"#);
    let project = load_project(parts.bytes()).unwrap();
    let (resolved, trace) = resolve_bounded_project_config_with_trace(&project).unwrap();
    assert_eq!(trace.events, expected_events());
    assert!(trace.usages[0].supported_used_filaments.is_empty());
    assert_eq!(trace.usages[1].supported_used_filaments, [0]);
    for call in &trace.normalize_fdm_2 {
        assert!(call.used_filament_count <= 1);
    }
    assert!(!trace.candidates[0].outer_wall_selectors.contains(&2));
    assert_eq!(resolved.usage.supported_used_filaments, [0]);
    assert_eq!(resolved.print_object_count, 2);
}

fn expected_events() -> Vec<PhaseEvent> {
    vec![
        PhaseEvent::NormalizeFdm1,
        PhaseEvent::NormalizeFdm2(NormalizeFdm2Pass::ColdFirst),
        PhaseEvent::Materialize(MaterializationPass::First),
        PhaseEvent::ValidateAndGroup(MaterializationPass::First),
        PhaseEvent::ResolveObjectShells,
        PhaseEvent::Usage(UsagePass::FirstPreRegion),
        PhaseEvent::NormalizeFdm2(NormalizeFdm2Pass::FirstLate),
        PhaseEvent::Candidates(CandidatePass::Final),
        PhaseEvent::Usage(UsagePass::FinalPostNormalize),
        PhaseEvent::ResolveViews,
    ]
}

fn two_z_group_fixture() -> ProjectParts {
    let mut parts = ProjectParts::fixture();
    parts.replace(
        "3D/3dmodel.model",
        "</build>",
        r#"<item objectid="2" transform="1 0 0 0 1 0 0 0 1 133.039205 115.992105 146" printable="1" auto_drop="1"/></build>"#,
    );
    parts.replace(
        "Metadata/model_settings.config",
        "</plate>",
        r#"<model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="1"/><metadata key="identify_id" value="902"/></model_instance></plate>"#,
    );
    parts.replace(
        "Metadata/model_settings.config",
        "</assemble>",
        r#"<assemble_item object_id="2" instance_id="1" transform="1 0 0 0 1 0 0 0 1 0 0 146" offset="0 0 0"/></assemble>"#,
    );
    parts
}

fn set_zero_raw_selectors(parts: &mut ProjectParts) {
    parts.replace(
        "Metadata/model_settings.config",
        r#"<metadata key="extruder" value="1"/>"#,
        r#"<metadata key="extruder" value="0"/>"#,
    );
    parts.replace(
        "Metadata/model_settings.config",
        r#"<part id="1" subtype="normal_part">"#,
        r#"<part id="1" subtype="normal_part"><metadata key="extruder" value="0"/>"#,
    );
}

fn set_zero_raw_and_two_feature_filaments(parts: &mut ProjectParts) {
    set_zero_raw_selectors(parts);
    parts.replace(
        "Metadata/model_settings.config",
        r#"<metadata key="extruder" value="0"/>"#,
        r#"<metadata key="extruder" value="0"/><metadata key="outer_wall_filament_id" value="1"/><metadata key="inner_wall_filament_id" value="2"/>"#,
    );
}
