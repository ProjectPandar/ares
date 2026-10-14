use crate::{
    ObjectOptions, OrcaBool, OrcaInt, ProjectInstance, ProjectObject, ProjectSettings,
    ProjectVolume, RegionOptions, SliceError, Transform3d, load_project,
    options::ObjectOptionOverrides,
    project::effective_config::types::{
        ResolvedLayerCandidate, ResolvedModelPartCandidate, ResolvedPrintObjectConfig,
        ResolvedProjectObject,
    },
};

use super::super::capabilities::validate as validate_capabilities;
use super::support::KsrArchive;

fn validate(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<(), SliceError> {
    validate_capabilities(source_objects, resolved_objects, false)
}

#[test]
fn task22a_capability_gates_each_named_feature() {
    let project = load_project(include_bytes!(
        "../../../../../tests/parity/object-layer-profiles/step.3mf"
    ))
    .unwrap();
    assert_eq!(
        validate(
            project.objects(),
            &[resolved(0, object_options(), Vec::new())]
        ),
        Ok(())
    );

    let ranged = source_object(Default::default(), Vec::new(), one_layer_height_range());
    assert_eq!(
        validate(&[ranged], &[resolved(0, object_options(), Vec::new())],),
        Ok(())
    );

    let source = source_object(Default::default(), Vec::new(), Vec::new());
    let object = with_object(|value| value.raft_layers = OrcaInt(1));
    // The raft gate is removed (raft milestone): raft_layers no longer
    // crosses the capability boundary.
    assert_eq!(
        validate(&[source], &[resolved(0, object, Vec::new())]),
        Ok(())
    );
}

#[test]
fn task22a_capability_gate_order_is_project_key_major() {
    let sources = [
        source_object(Default::default(), Vec::new(), Vec::new()),
        source_object(Default::default(), Vec::new(), one_layer_height_range()),
    ];
    let mut first = with_object(|value| {
        value.raft_layers = OrcaInt(1);
        value.enable_support = OrcaBool(true);
        value.enforce_support_layers = OrcaInt(1);
        value.precise_z_height = OrcaBool(true);
    });
    let mut resolved_objects = vec![
        resolved(0, first.clone(), Vec::new()),
        resolved(1, object_options(), Vec::new()),
    ];

    let mut profiled = sources.clone();
    profiled[0].set_layer_height_profile(vec![0.0, 0.2, 0.5, 0.1, 1.0, 0.2]);
    assert_eq!(validate(&profiled, &resolved_objects), Ok(()));
    assert_eq!(validate(&sources, &resolved_objects), Ok(()));

    let sources = [
        source_object(Default::default(), Vec::new(), Vec::new()),
        source_object(Default::default(), Vec::new(), Vec::new()),
    ];
    // The raft gate is removed: raft_layers=1 validates clean.
    assert_eq!(validate(&sources, &resolved_objects), Ok(()));
    first.raft_layers = OrcaInt(0);
    resolved_objects[0].object = first;
    assert_eq!(validate(&sources, &resolved_objects), Ok(()));
}

#[test]
fn task22o18_global_spiral_crosses_the_capability_boundary_after_earlier_gates() {
    assert_eq!(validate_capabilities(&[], &[], true), Ok(()));

    let source = source_object(Default::default(), Vec::new(), Vec::new());
    let object = with_object(|value| value.raft_layers = OrcaInt(1));
    // The raft gate is removed: only spiral_mode gates here.
    assert_eq!(
        validate_capabilities(&[source], &[resolved(0, object, Vec::new())], true),
        Ok(())
    );
}

#[test]
fn task22o17_support_options_cross_the_early_capability_boundary() {
    let source = source_object(Default::default(), Vec::new(), Vec::new());
    let object = with_object(|value| {
        value.enable_support = OrcaBool(true);
        value.enforce_support_layers = OrcaInt(1);
    });
    assert_eq!(
        validate(&[source], &[resolved(0, object, Vec::new())]),
        Ok(())
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

fn resolved(
    source_object_index: usize,
    object: ObjectOptions,
    regions: Vec<RegionOptions>,
) -> ResolvedProjectObject {
    ResolvedProjectObject {
        source_object_index,
        all_regions: regions.clone(),
        object,
        print_objects: vec![ResolvedPrintObjectConfig {
            transform: Transform3d::IDENTITY,
            instance_indices: vec![0],
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
