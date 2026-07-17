use crate::{
    ObjectOptions, OrcaBool, OrcaInt, Point3d, ProjectInstance, ProjectMesh, ProjectObject,
    ProjectSettings, ProjectVolume, ProjectVolumeType, RegionOptions, SliceError, Transform3d,
    load_project,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
    project::effective_config::types::{
        ResolvedLayerCandidate, ResolvedModelPartCandidate, ResolvedPrintObjectConfig,
        ResolvedProjectObject,
    },
    slice_project,
};

use super::super::capabilities::validate;
use super::support::{KsrArchive, metadata};

#[test]
fn task22a_capability_gates_each_named_feature() {
    assert_unsupported(validate(true, &[], &[]), "layer_height_profile");

    let ranged = source_object(Default::default(), Vec::new(), one_layer_height_range());
    assert_unsupported(
        validate(
            false,
            &[ranged],
            &[resolved(0, object_options(), Vec::new())],
        ),
        "layer_height",
    );

    for (key, object) in [
        (
            "raft_layers",
            with_object(|value| value.raft_layers = OrcaInt(1)),
        ),
        (
            "enable_support",
            with_object(|value| value.enable_support = OrcaBool(true)),
        ),
        (
            "enforce_support_layers",
            with_object(|value| value.enforce_support_layers = OrcaInt(1)),
        ),
        (
            "precise_z_height",
            with_object(|value| value.precise_z_height = OrcaBool(true)),
        ),
    ] {
        let source = source_object(Default::default(), Vec::new(), Vec::new());
        assert_unsupported(
            validate(false, &[source], &[resolved(0, object, Vec::new())]),
            key,
        );
    }

    let source = source_object(Default::default(), Vec::new(), Vec::new());
    assert_unsupported(
        validate(
            false,
            &[source],
            &[resolved(0, object_options(), vec![region(true)])],
        ),
        "zaa_enabled",
    );
}

#[test]
fn task22a_capability_gate_order_is_project_key_major() {
    let true_modifier = modifier(Some(true), 100.0);
    let sources = [
        source_object(Default::default(), vec![true_modifier], Vec::new()),
        source_object(Default::default(), Vec::new(), one_layer_height_range()),
    ];
    let mut first = with_object(|value| {
        value.raft_layers = OrcaInt(1);
        value.enable_support = OrcaBool(true);
        value.enforce_support_layers = OrcaInt(1);
        value.precise_z_height = OrcaBool(true);
    });
    let mut resolved_objects = vec![
        resolved(0, first.clone(), vec![region(true)]),
        resolved(1, object_options(), Vec::new()),
    ];

    assert_unsupported(
        validate(true, &sources, &resolved_objects),
        "layer_height_profile",
    );
    assert_unsupported(validate(false, &sources, &resolved_objects), "layer_height");

    let sources = [
        source_object(
            Default::default(),
            vec![modifier(Some(true), 100.0)],
            Vec::new(),
        ),
        source_object(Default::default(), Vec::new(), Vec::new()),
    ];
    assert_unsupported(validate(false, &sources, &resolved_objects), "raft_layers");
    first.raft_layers = OrcaInt(0);
    resolved_objects[0].object = first.clone();
    assert_unsupported(
        validate(false, &sources, &resolved_objects),
        "enable_support",
    );
    first.enable_support = OrcaBool(false);
    resolved_objects[0].object = first.clone();
    assert_unsupported(
        validate(false, &sources, &resolved_objects),
        "enforce_support_layers",
    );
    first.enforce_support_layers = OrcaInt(0);
    resolved_objects[0].object = first.clone();
    assert_unsupported(
        validate(false, &sources, &resolved_objects),
        "precise_z_height",
    );
    first.precise_z_height = OrcaBool(false);
    resolved_objects[0].object = first;
    assert_unsupported(validate(false, &sources, &resolved_objects), "zaa_enabled");
}

#[test]
fn task22a_zaa_gate_scans_candidate_and_nonintersecting_modifier() {
    let source = source_object(Default::default(), Vec::new(), Vec::new());
    assert_unsupported(
        validate(
            false,
            &[source],
            &[resolved(0, object_options(), vec![region(true)])],
        ),
        "zaa_enabled",
    );

    let source = source_object(
        Default::default(),
        vec![model_part(0.0), modifier(Some(true), 100.0)],
        Vec::new(),
    );
    assert_unsupported(
        validate(
            false,
            &[source],
            &[resolved(0, object_options(), vec![region(false)])],
        ),
        "zaa_enabled",
    );
}

#[test]
fn task22a_zaa_false_modifier_is_supported() {
    for zaa_enabled in [None, Some(false)] {
        let source = source_object(
            Default::default(),
            vec![modifier(zaa_enabled, 100.0)],
            Vec::new(),
        );
        assert_eq!(
            validate(
                false,
                &[source],
                &[resolved(0, object_options(), vec![region(false)])],
            ),
            Ok(())
        );
    }
}

#[tokio::test]
async fn task22a_zaa_gate_runs_after_config_writer() {
    let mut supported_writer = KsrArchive::new();
    supported_writer.replace(
        "Metadata/model_settings.config",
        r#"<part id="1" subtype="normal_part">"#,
        r#"<part id="1" subtype="modifier_part"><metadata key="zaa_enabled" value="1"/>"#,
    );
    let mut invalid_writer = supported_writer.clone();
    invalid_writer.invalidate_flush_matrix();

    assert_eq!(
        slice_project(invalid_writer.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::InvalidInput(
            "Flush volumes matrix do not match to the correct size!".to_owned()
        )
    );
    assert_eq!(
        slice_project(supported_writer.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("zaa_enabled".to_owned())
    );
}

fn assert_unsupported(result: Result<(), SliceError>, key: &str) {
    assert_eq!(
        result.unwrap_err(),
        SliceError::UnsupportedProjectFeature(key.to_owned())
    );
}

fn with_object(update: impl FnOnce(&mut ObjectOptions)) -> ObjectOptions {
    let mut options = object_options();
    update(&mut options);
    options
}

fn object_options() -> ObjectOptions {
    ObjectOptions::from_base(&ProjectSettings::default().process.object)
}

fn region(zaa_enabled: bool) -> RegionOptions {
    let mut region = RegionOptions::from_base(&ProjectSettings::default().process.region);
    region.zaa_enabled = OrcaBool(zaa_enabled);
    region
}

fn resolved(
    source_object_index: usize,
    object: ObjectOptions,
    regions: Vec<RegionOptions>,
) -> ResolvedProjectObject {
    ResolvedProjectObject {
        source_object_index,
        object,
        print_objects: vec![ResolvedPrintObjectConfig {
            transform: Transform3d::IDENTITY,
        }],
        layer_candidates: vec![ResolvedLayerCandidate {
            min_z: 0.0,
            max_z: 1.0,
            source_range_index: None,
            model_parts: regions
                .into_iter()
                .enumerate()
                .map(|(volume_index, region)| ResolvedModelPartCandidate {
                    volume_index,
                    region,
                })
                .collect(),
        }],
    }
}

fn source_object(
    object_overrides: ObjectOptionOverrides,
    volumes: Vec<ProjectVolume>,
    ranges: Vec<crate::LayerConfigRange>,
) -> ProjectObject {
    let mut object = ProjectObject::new(
        "synthetic.model".to_owned(),
        1,
        (
            "object".to_owned(),
            String::new(),
            object_overrides,
            Default::default(),
        ),
        volumes,
        vec![ProjectInstance::new(
            [1, 0, 1_000],
            true,
            false,
            Transform3d::IDENTITY,
        )],
    );
    object.set_layer_config_ranges(ranges);
    object
}

fn one_layer_height_range() -> Vec<crate::LayerConfigRange> {
    let mut archive = KsrArchive::new();
    archive.insert_text(
        "Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1"><range min_z="0" max_z="1"><option opt_key="layer_height">0.18</option></range></object></objects>"#,
    );
    load_project(archive.bytes()).unwrap().objects()[0]
        .layer_config_ranges()
        .to_vec()
}

fn model_part(z: f64) -> ProjectVolume {
    volume(ProjectVolumeType::ModelPart, None, z)
}

fn modifier(zaa_enabled: Option<bool>, z: f64) -> ProjectVolume {
    volume(ProjectVolumeType::ParameterModifier, zaa_enabled, z)
}

fn volume(volume_type: ProjectVolumeType, zaa_enabled: Option<bool>, z: f64) -> ProjectVolume {
    ProjectVolume::new(
        "synthetic.model".to_owned(),
        1,
        ProjectMesh::new(
            vec![
                Point3d::new(0.0, 0.0, z),
                Point3d::new(1.0, 0.0, z),
                Point3d::new(0.0, 1.0, z),
            ],
            vec![[0, 1, 2]],
        ),
        Transform3d::IDENTITY,
        (
            "volume".to_owned(),
            volume_type,
            RegionOptionOverrides {
                zaa_enabled: zaa_enabled.map(OrcaBool),
                ..Default::default()
            },
            Transform3d::IDENTITY,
        ),
    )
}
