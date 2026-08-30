use crate::{
    LayerConfigRange, ObjectOptions, OrcaFloat, OrcaInt, Percent, Point3d, ProcessBrimType,
    ProjectInstance, ProjectMesh, ProjectObject, ProjectSettings, ProjectVolume, ProjectVolumeType,
    RegionOptions, SliceError, load_project,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
    project::{
        effective_config::{
            ValidatedMaterializedProject,
            candidates::{resolve_project_candidates, resolve_project_objects},
            grouping::group_print_object_transforms,
            types::{ResolvedLayerCandidate, ResolvedPrintObjectConfig},
        },
        transform::Transform3d,
    },
};

use super::{support::ProjectParts, valid_settings};

#[test] #[rustfmt::skip]
fn reverse_input_uses_first_sorted_group_and_one_shared_candidate_set() {
    let objects = [object(
        Default::default(), Default::default(),
        vec![volume(10, ProjectVolumeType::ModelPart, 0.0, true, Default::default())],
        vec![z_translation(100.0), Transform3d::IDENTITY],
        layer_ranges(r#"<range min_z="0" max_z="1"><option opt_key="outer_wall_filament_id">1</option></range><range min_z="100" max_z="101"><option opt_key="outer_wall_filament_id">2</option></range>"#),
    )];
    let settings = settings();
    let groups = group_print_object_transforms(&objects);
    let resolved = resolve_project_candidates(&settings, validated(), &objects, &groups).unwrap();

    assert_eq!(resolved.len(), 1);
    let object = &resolved[0];
    let _: &ObjectOptions = &object.object;
    let _: &[ResolvedPrintObjectConfig] = &object.print_objects;
    let _: &[ResolvedLayerCandidate] = &object.layer_candidates;
    assert_eq!(object.print_objects.len(), 2);
    assert!(object.print_objects[0].transform.fixed_order_equal(Transform3d::IDENTITY));
    assert!(object.print_objects[1].transform.fixed_order_equal(z_translation(100.0)));
    assert_eq!(
        object.layer_candidates.iter().map(|candidate| candidate.source_range_index).collect::<Vec<_>>(),
        vec![Some(0), None, Some(1), None]
    );
    let first = object.layer_candidates.iter().find(|candidate| candidate.source_range_index == Some(0)).unwrap();
    assert_eq!((first.min_z, first.max_z), (0.0, 1.0));
    assert_eq!(first.model_parts.len(), 1);
    assert_eq!(first.model_parts[0].volume_index, 0);
    assert_eq!(first.model_parts[0].region.outer_wall_filament_id, OrcaInt(1));
    assert!(object.layer_candidates.iter().find(|candidate| candidate.source_range_index == Some(1)).unwrap().model_parts.is_empty());
    assert_eq!(
        object.layer_candidates.iter().map(|candidate| candidate.model_parts.len()).sum::<usize>(),
        1
    );
}

#[test] #[rustfmt::skip]
fn source_objects_keep_separate_candidate_ownership() {
    let first_region = RegionOptionOverrides { wall_loops: Some(OrcaInt(2)), ..Default::default() };
    let second_region = RegionOptionOverrides { wall_loops: Some(OrcaInt(3)), ..Default::default() };
    let objects = [
        object(Default::default(), first_region, vec![model_part(10)], vec![Transform3d::IDENTITY], Vec::new()),
        object(Default::default(), second_region, vec![model_part(20)], vec![Transform3d::IDENTITY], Vec::new()),
    ];
    let resolved = resolve(&settings(), &objects).unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(only_region(&resolved[0]).wall_loops, OrcaInt(2));
    assert_eq!(only_region(&resolved[1]).wall_loops, OrcaInt(3));
}

#[test]
fn task22a_source_object_index_survives_filtered_object() {
    let objects = [
        object(
            Default::default(),
            Default::default(),
            vec![model_part(10)],
            Vec::new(),
            Vec::new(),
        ),
        object(
            Default::default(),
            Default::default(),
            vec![model_part(20)],
            vec![Transform3d::IDENTITY],
            Vec::new(),
        ),
    ];
    let groups = group_print_object_transforms(&objects);

    let shells = resolve_project_objects(&settings(), validated(), &objects, &groups).unwrap();
    assert_eq!(shells.len(), 1);
    assert_eq!(shells[0].source_object_index, 1);

    let candidates =
        resolve_project_candidates(&settings(), validated(), &objects, &groups).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].source_object_index, 1);
}

#[test] #[rustfmt::skip]
fn object_without_groups_skips_all_candidate_source_checks() {
    let object_options = ObjectOptionOverrides {
        brim_type: Some(ProcessBrimType::Painted), brim_width: Some(OrcaFloat(0.0)), ..Default::default()
    };
    let modifier = RegionOptionOverrides { wall_loops: Some(OrcaInt(9)), ..Default::default() };
    let objects = [object(object_options, Default::default(),
        vec![volume(1, ProjectVolumeType::ParameterModifier, 0.0, true, modifier)], Vec::new(), Vec::new())];
    assert!(resolve(&settings(), &objects).unwrap().is_empty());
}

#[test] #[rustfmt::skip]
fn object_support_selectors_clamp_only_values_above_logical_count() {
    let objects = [
        object_with_support(3, 4),
        object_with_support(0, 0),
        object_with_support(2, 2),
    ];

    let resolved = resolve(&settings(), &objects).unwrap();

    assert_eq!(resolved.iter().map(|object| (
        object.object.support_filament, object.object.support_interface_filament,
    )).collect::<Vec<_>>(),
        vec![
            (OrcaInt(1), OrcaInt(1)),
            (OrcaInt(0), OrcaInt(0)),
            (OrcaInt(2), OrcaInt(2)),
        ]
    );
}

#[test]
fn candidate_region_feature_selectors_clamp_against_logical_count() {
    let mut settings = settings();
    settings.process.region.outer_wall_filament_id = OrcaInt(3);
    settings.process.region.inner_wall_filament_id = OrcaInt(2);
    let objects = [object(
        Default::default(),
        Default::default(),
        vec![model_part(1)],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];

    let resolved = resolve(&settings, &objects).unwrap();
    let region = only_region(&resolved[0]);

    assert_eq!(region.outer_wall_filament_id, OrcaInt(1));
    assert_eq!(region.inner_wall_filament_id, OrcaInt(2));
    assert_eq!(settings.process.region.outer_wall_filament_id, OrcaInt(3));
    assert_eq!(settings.process.region.inner_wall_filament_id, OrcaInt(2));
}

#[test] #[rustfmt::skip]
fn region_precedence_is_process_object_volume_none_material_then_layer() {
    let mut settings = settings();
    settings.process.region.wall_loops = OrcaInt(1);
    settings.process.region.top_shell_layers = OrcaInt(1);
    settings.process.region.bottom_shell_layers = OrcaInt(1);
    settings.process.region.sparse_infill_density = Percent(11.0);
    let object_region = RegionOptionOverrides {
        wall_loops: Some(OrcaInt(2)), top_shell_layers: Some(OrcaInt(2)),
        bottom_shell_layers: Some(OrcaInt(2)), ..Default::default()
    };
    let volume_region = RegionOptionOverrides {
        wall_loops: Some(OrcaInt(3)), top_shell_layers: Some(OrcaInt(3)), ..Default::default()
    };
    let objects = [object(Default::default(), object_region,
        vec![volume(1, ProjectVolumeType::ModelPart, 0.0, true, volume_region)],
        vec![Transform3d::IDENTITY],
        layer_ranges(r#"<range min_z="0" max_z="1"><option opt_key="wall_loops">4</option></range>"#))];
    let resolved = resolve(&settings, &objects).unwrap();
    let region = only_region(&resolved[0]);
    assert_eq!(region.wall_loops, OrcaInt(4));
    assert_eq!(region.top_shell_layers, OrcaInt(3));
    assert_eq!(region.bottom_shell_layers, OrcaInt(2));
    assert_eq!(region.sparse_infill_density, Percent(11.0));
}

#[test] #[rustfmt::skip]
fn only_nonempty_model_parts_receive_candidates() {
    let modifier = RegionOptionOverrides { bridge_angle: Some(OrcaFloat(12.0)), ..Default::default() };
    let objects = [object(
        Default::default(),
        Default::default(),
        vec![
            volume(1, ProjectVolumeType::NegativeVolume, 0.0, true, Default::default()),
            volume(2, ProjectVolumeType::ModelPart, 0.0, false, Default::default()),
            volume(3, ProjectVolumeType::ModelPart, 0.0, true, Default::default()),
            volume(4, ProjectVolumeType::SupportEnforcer, 0.0, true, Default::default()),
            volume(5, ProjectVolumeType::SupportBlocker, 0.0, true, Default::default()),
            volume(6, ProjectVolumeType::ParameterModifier, 0.0, true, modifier),
        ],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let resolved = resolve(&settings(), &objects).unwrap();
    let model_parts = &resolved[0].layer_candidates[0].model_parts;
    assert_eq!(model_parts.len(), 1);
    assert_eq!(model_parts[0].volume_index, 2);
}

#[test] #[rustfmt::skip]
fn usage_affecting_modifier_keys_are_rejected_in_fixed_order() {
    let cases = [
        ("wall_loops", RegionOptionOverrides { wall_loops: Some(OrcaInt(2)), ..Default::default() }),
        ("sparse_infill_density", RegionOptionOverrides { sparse_infill_density: Some(Percent(20.0)), ..Default::default() }),
        ("top_shell_layers", RegionOptionOverrides { top_shell_layers: Some(OrcaInt(2)), ..Default::default() }),
        ("bottom_shell_layers", RegionOptionOverrides { bottom_shell_layers: Some(OrcaInt(2)), ..Default::default() }),
        ("sparse_infill_filament_id", RegionOptionOverrides { sparse_infill_filament_id: Some(OrcaInt(2)), ..Default::default() }),
        ("internal_solid_filament_id", RegionOptionOverrides { internal_solid_filament_id: Some(OrcaInt(2)), ..Default::default() }),
        ("top_surface_filament_id", RegionOptionOverrides { top_surface_filament_id: Some(OrcaInt(2)), ..Default::default() }),
        ("bottom_surface_filament_id", RegionOptionOverrides { bottom_surface_filament_id: Some(OrcaInt(2)), ..Default::default() }),
        ("outer_wall_filament_id", RegionOptionOverrides { outer_wall_filament_id: Some(OrcaInt(2)), ..Default::default() }),
        ("inner_wall_filament_id", RegionOptionOverrides { inner_wall_filament_id: Some(OrcaInt(2)), ..Default::default() }),
    ];
    for (key, overrides) in cases {
        assert_eq!(modifier_error(overrides), SliceError::UnsupportedProjectFeature(key.to_owned()));
    }
    let combined = RegionOptionOverrides {
        wall_loops: Some(OrcaInt(2)),
        sparse_infill_density: Some(Percent(20.0)),
        top_shell_layers: Some(OrcaInt(2)),
        bottom_shell_layers: Some(OrcaInt(2)),
        sparse_infill_filament_id: Some(OrcaInt(2)),
        internal_solid_filament_id: Some(OrcaInt(2)),
        top_surface_filament_id: Some(OrcaInt(2)),
        bottom_surface_filament_id: Some(OrcaInt(2)),
        outer_wall_filament_id: Some(OrcaInt(2)),
        inner_wall_filament_id: Some(OrcaInt(2)),
        ..Default::default()
    };
    assert_eq!(
        modifier_error(combined),
        SliceError::UnsupportedProjectFeature("wall_loops".to_owned())
    );
}

#[test] #[rustfmt::skip]
fn zero_width_painted_brim_is_an_inactive_configuration() {
    assert!(resolve(&settings(), &[object_with_brim(ProcessBrimType::Painted, 0.0)]).is_ok());
    assert!(resolve(&settings(), &[object_with_brim(ProcessBrimType::Painted, 1.0)]).is_ok());
    assert!(resolve(&settings(), &[object_with_brim(ProcessBrimType::AutoBrim, 0.0)]).is_ok());
}

#[rustfmt::skip]
fn resolve(settings: &ProjectSettings, objects: &[ProjectObject]) -> Result<Vec<crate::project::effective_config::types::ResolvedProjectObject>, SliceError> {
    let groups = group_print_object_transforms(objects);
    resolve_project_candidates(settings, validated(), objects, &groups)
}

#[rustfmt::skip]
fn validated() -> ValidatedMaterializedProject {
    ValidatedMaterializedProject { physical_extruder_count: 4, logical_filament_count: 2 }
}

fn settings() -> ProjectSettings {
    valid_settings(4, 2)
}

#[rustfmt::skip]
fn only_region(object: &crate::project::effective_config::types::ResolvedProjectObject) -> &RegionOptions {
    &object.layer_candidates.iter().flat_map(|candidate| &candidate.model_parts).next().unwrap().region
}

#[rustfmt::skip]
fn object_with_support(support: i32, interface: i32) -> ProjectObject {
    let overrides = ObjectOptionOverrides {
        support_filament: Some(OrcaInt(support)),
        support_interface_filament: Some(OrcaInt(interface)),
        ..Default::default()
    };
    object(overrides, Default::default(), Vec::new(), vec![Transform3d::IDENTITY], Vec::new())
}

#[rustfmt::skip]
fn object_with_brim(brim_type: ProcessBrimType, width: f64) -> ProjectObject {
    let overrides = ObjectOptionOverrides {
        brim_type: Some(brim_type),
        brim_width: Some(OrcaFloat(width)),
        ..Default::default()
    };
    object(overrides, Default::default(), Vec::new(), vec![Transform3d::IDENTITY], Vec::new())
}

#[rustfmt::skip]
fn modifier_error(overrides: RegionOptionOverrides) -> SliceError {
    let object = object(Default::default(), Default::default(),
        vec![volume(1, ProjectVolumeType::ParameterModifier, 0.0, true, overrides)],
        vec![Transform3d::IDENTITY], Vec::new());
    resolve(&settings(), &[object]).unwrap_err()
}

#[rustfmt::skip]
fn model_part(id: u32) -> ProjectVolume {
    volume(id, ProjectVolumeType::ModelPart, 0.0, true, Default::default())
}

#[rustfmt::skip]
fn volume(
    id: u32,
    volume_type: ProjectVolumeType,
    z: f64,
    nonempty: bool,
    overrides: RegionOptionOverrides,
) -> ProjectVolume {
    let vertices = vec![Point3d::new(0.0, 0.0, z), Point3d::new(1.0, 0.0, z), Point3d::new(0.0, 1.0, z)];
    let triangles = if nonempty { vec![[0, 1, 2]] } else { Vec::new() };
    ProjectVolume::new(
        "synthetic.model".to_owned(), id, ProjectMesh::new(vertices, triangles), Transform3d::IDENTITY,
        (format!("volume-{id}"), volume_type, overrides, Transform3d::IDENTITY),
    )
}

#[rustfmt::skip]
fn object(
    object_overrides: ObjectOptionOverrides,
    region_overrides: RegionOptionOverrides,
    volumes: Vec<ProjectVolume>,
    transforms: Vec<Transform3d>,
    ranges: Vec<LayerConfigRange>,
) -> ProjectObject {
    let id = 1;
    let instances = transforms.into_iter().enumerate().map(|(index, transform)| {
        ProjectInstance::new([id, u32::try_from(index).unwrap(), 1_000 + u32::try_from(index).unwrap()], true, false, transform)
    }).collect();
    let mut object = ProjectObject::new(
        "synthetic.model".to_owned(), id,
        (format!("object-{id}"), String::new(), object_overrides, region_overrides), volumes, instances,
    );
    object.set_layer_config_ranges(ranges);
    object
}

#[rustfmt::skip]
fn layer_ranges(body: &str) -> Vec<LayerConfigRange> {
    let mut parts = ProjectParts::valid();
    parts.insert_text("Metadata/layer_config_ranges.xml", &format!(r#"<objects><object id="1">{body}</object></objects>"#));
    load_project(parts.bytes()).unwrap().objects()[0].layer_config_ranges().to_vec()
}

fn z_translation(z: f64) -> Transform3d {
    Transform3d::parse_3mf(&format!("1 0 0 0 1 0 0 0 1 0 0 {z}")).unwrap()
}
