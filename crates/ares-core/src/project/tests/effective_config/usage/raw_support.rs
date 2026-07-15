use crate::{
    OrcaBool, OrcaInt, Percent, ProjectVolumeType,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
    project::transform::Transform3d,
};

use super::{
    base_region, collect, layer_ranges, object_options, printable_source, resolve_candidates,
    resolved_object, settings, source_object, volume,
};

#[test]
fn raw_model_volume_fallback_includes_only_supported_volume_types() {
    let settings = settings(4);
    let objects = [source_object(
        Default::default(),
        RegionOptionOverrides {
            extruder: Some(OrcaInt(2)),
            ..Default::default()
        },
        vec![
            volume(ProjectVolumeType::ModelPart, Some(3), 0.0, true),
            volume(ProjectVolumeType::ParameterModifier, Some(0), 0.0, true),
            volume(ProjectVolumeType::ModelPart, None, 0.0, true),
            volume(ProjectVolumeType::NegativeVolume, Some(4), 0.0, true),
            volume(ProjectVolumeType::SupportEnforcer, Some(4), 0.0, true),
            volume(ProjectVolumeType::SupportBlocker, Some(4), 0.0, true),
        ],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let resolved = [resolved_object(object_options(&settings), Vec::new())];

    assert_eq!(
        collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
        vec![1, 2]
    );
}

#[test]
fn absent_object_and_volume_selectors_default_to_one_but_explicit_zero_does_not() {
    let settings = settings(2);
    let explicit_zero = [source_object(
        Default::default(),
        RegionOptionOverrides {
            extruder: Some(OrcaInt(0)),
            ..Default::default()
        },
        vec![
            volume(ProjectVolumeType::ModelPart, None, 0.0, true),
            volume(ProjectVolumeType::ParameterModifier, Some(0), 0.0, true),
        ],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let zero_resolved = [resolved_object(object_options(&settings), Vec::new())];
    assert!(
        collect(&settings, &settings, &explicit_zero, &zero_resolved)
            .supported_used_filaments
            .is_empty()
    );

    let absent = [source_object(
        Default::default(),
        Default::default(),
        vec![volume(ProjectVolumeType::ModelPart, None, 0.0, true)],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let absent_resolved = [resolved_object(object_options(&settings), Vec::new())];
    assert_eq!(
        collect(&settings, &settings, &absent, &absent_resolved).supported_used_filaments,
        vec![0]
    );
}

#[test]
fn zero_volume_selector_with_no_object_selector_defaults_to_one() {
    let settings = settings(2);
    let objects = [source_object(
        Default::default(),
        Default::default(),
        vec![volume(ProjectVolumeType::ModelPart, Some(0), 0.0, true)],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let resolved = [resolved_object(object_options(&settings), Vec::new())];

    assert_eq!(
        collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
        vec![0]
    );
}

#[test]
fn nonintersecting_layer_feature_is_absent_while_its_raw_extruder_participates() {
    let mut settings = settings(3);
    settings.process.region.wall_loops = OrcaInt(1);
    settings.process.region.sparse_infill_density = Percent(0.0);
    settings.process.region.top_shell_layers = OrcaInt(0);
    settings.process.region.bottom_shell_layers = OrcaInt(0);
    let objects = [source_object(
        Default::default(),
        Default::default(),
        vec![volume(ProjectVolumeType::ModelPart, None, 0.0, true)],
        vec![Transform3d::IDENTITY],
        layer_ranges(
            r#"<range min_z="100" max_z="101"><option opt_key="outer_wall_filament_id">3</option><option opt_key="extruder">2</option></range>"#,
        ),
    )];
    let resolved = resolve_candidates(&settings, 3, &objects);
    let usage = collect(&settings, &settings, &objects, &resolved);

    assert_eq!(usage.supported_used_filaments, vec![0, 1]);
    assert!(!usage.supported_used_filaments.contains(&2));
}

#[test]
fn object_without_a_printable_group_contributes_no_usage_source() {
    let settings = settings(4);
    let objects = [source_object(
        Default::default(),
        RegionOptionOverrides {
            extruder: Some(OrcaInt(4)),
            ..Default::default()
        },
        vec![volume(ProjectVolumeType::ModelPart, Some(3), 0.0, true)],
        Vec::new(),
        layer_ranges(r#"<range min_z="0" max_z="1"><option opt_key="extruder">2</option></range>"#),
    )];
    let mut options = object_options(&settings);
    options.brim_type = crate::ProcessBrimType::AutoBrim;
    options.enable_support = OrcaBool(true);
    options.support_filament = OrcaInt(4);
    let mut region = base_region(&settings);
    region.wall_loops = OrcaInt(1);
    region.outer_wall_filament_id = OrcaInt(1);
    let resolved = [resolved_object(options, vec![region])];

    assert!(
        collect(&settings, &settings, &objects, &resolved)
            .supported_used_filaments
            .is_empty()
    );
}

#[test]
fn resolved_sources_stay_aligned_after_an_object_without_a_group() {
    let settings = settings(3);
    let objects = [
        source_object(
            Default::default(),
            Default::default(),
            vec![volume(ProjectVolumeType::ModelPart, Some(3), 0.0, true)],
            Vec::new(),
            Vec::new(),
        ),
        source_object(
            Default::default(),
            Default::default(),
            vec![volume(ProjectVolumeType::ModelPart, Some(2), 0.0, true)],
            vec![Transform3d::IDENTITY],
            Vec::new(),
        ),
    ];
    let resolved = resolve_candidates(&settings, 3, &objects);

    assert_eq!(resolved.len(), 1);
    assert_eq!(
        collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
        vec![1]
    );
}

#[test]
fn support_routes_only_when_enabled_by_support_enforcement_or_raft() {
    let cases = [
        (false, 0, 0, vec![]),
        (true, 0, 0, vec![1, 2]),
        (false, 1, 0, vec![1, 2]),
        (false, 0, 1, vec![1, 2]),
    ];

    for (enabled, enforced, raft, expected) in cases {
        let settings = settings(3);
        let mut options = object_options(&settings);
        options.enable_support = OrcaBool(enabled);
        options.enforce_support_layers = OrcaInt(enforced);
        options.raft_layers = OrcaInt(raft);
        options.support_filament = OrcaInt(2);
        options.support_interface_filament = OrcaInt(3);
        let objects = [printable_source()];
        let resolved = [resolved_object(options, Vec::new())];

        assert_eq!(
            collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
            expected
        );
    }
}

#[test]
fn current_support_selector_preserves_the_complete_object_set() {
    let settings = settings(3);
    let objects = [
        source_object(
            Default::default(),
            Default::default(),
            vec![volume(ProjectVolumeType::ModelPart, Some(2), 0.0, true)],
            vec![Transform3d::IDENTITY],
            Vec::new(),
        ),
        source_object(
            Default::default(),
            Default::default(),
            vec![volume(ProjectVolumeType::ModelPart, Some(3), 0.0, true)],
            vec![Transform3d::IDENTITY],
            Vec::new(),
        ),
    ];
    let mut supported = object_options(&settings);
    supported.enable_support = OrcaBool(true);
    supported.support_filament = OrcaInt(0);
    supported.support_interface_filament = OrcaInt(0);
    let resolved = [
        resolved_object(supported, Vec::new()),
        resolved_object(object_options(&settings), Vec::new()),
    ];

    assert_eq!(
        collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
        vec![1, 2]
    );
}

#[test]
fn mixed_current_and_positive_support_selectors_keep_both_routes() {
    let settings = settings(3);
    let objects = [source_object(
        Default::default(),
        Default::default(),
        vec![
            volume(ProjectVolumeType::ModelPart, Some(1), 0.0, true),
            volume(ProjectVolumeType::ParameterModifier, Some(2), 0.0, true),
        ],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let mut options = object_options(&settings);
    options.enable_support = OrcaBool(true);
    options.support_filament = OrcaInt(0);
    options.support_interface_filament = OrcaInt(3);
    let resolved = [resolved_object(options, Vec::new())];

    assert_eq!(
        collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
        vec![0, 1, 2]
    );
}

#[test]
fn resolved_support_selectors_use_the_existing_clamp_to_one() {
    let settings = settings(2);
    let objects = [source_object(
        ObjectOptionOverrides {
            enable_support: Some(OrcaBool(true)),
            support_filament: Some(OrcaInt(4)),
            support_interface_filament: Some(OrcaInt(4)),
            ..Default::default()
        },
        Default::default(),
        Vec::new(),
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let resolved = resolve_candidates(&settings, 2, &objects);

    assert_eq!(
        collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
        vec![0]
    );
}
