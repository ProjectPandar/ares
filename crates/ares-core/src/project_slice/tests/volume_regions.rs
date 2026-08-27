use crate::project_slice::ProjectSource;
use crate::{
    OrcaFloat, OrcaInt, Point3d, ProjectMesh, ProjectVolume, ProjectVolumeType, RegionOptions,
    SliceError, Transform3d,
    mesh_slicer::SlicingMode,
    options::{FilamentRegionSourceOptions, RegionOptionOverrides},
    project::effective_config::types::ResolvedModelPartCandidate,
};

use super::{
    super::{
        PreparedPostClosing,
        closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
        prepare_post_simplification,
        state::prepare_project_slice,
        volume_bounds::build_volume_bounds,
        volume_regions::{
            VolumeRegionGraph, build_volume_region_graph, model_part_for_source_index,
        },
    },
    region_fixture::modifier_projects,
    support::{KsrArchive, object, plan, region, resolved_object},
};

type RecordFact = (
    usize,
    u32,
    ProjectVolumeType,
    Option<usize>,
    Option<usize>,
    usize,
);

#[test]
fn task22j_single_model_part_creates_region_zero() {
    let graph = graph(
        vec![volume(9, ProjectVolumeType::ModelPart, 0.0, 1.0, None)],
        &[(0, 17)],
        vec![(0, region())],
    );
    assert_eq!(graph.all_regions, vec![region()]);
    #[rustfmt::skip]
    assert_eq!(facts(&graph), vec![(0, 17, ProjectVolumeType::ModelPart, None, Some(0), 0)]);
}

#[test]
fn task22j_model_parts_dedupe_a_b_a_in_source_order() {
    let a = region();
    let mut b = a.clone();
    b.bridge_angle = OrcaFloat(19.0);
    let volumes = [3, 1, 2]
        .map(|id| volume(id, ProjectVolumeType::ModelPart, 0.0, 1.0, None))
        .to_vec();
    let graph = graph(
        volumes,
        &[(0, 9), (1, 3), (2, 7)],
        vec![(0, a.clone()), (1, b.clone()), (2, a.clone())],
    );
    assert_eq!(graph.all_regions, vec![a, b]);
    #[rustfmt::skip]
    assert_eq!(facts(&graph), vec![(0, 9, ProjectVolumeType::ModelPart, None, Some(0), 2), (1, 3, ProjectVolumeType::ModelPart, None, Some(1), 0), (2, 7, ProjectVolumeType::ModelPart, None, Some(0), 1)]);
}

#[test]
fn task22j_negative_records_have_no_region_or_parent() {
    let volumes = [
        ProjectVolumeType::ModelPart,
        ProjectVolumeType::NegativeVolume,
    ]
    .map(|kind| volume(1, kind, 0.0, 1.0, None))
    .to_vec();
    let graph = graph(volumes, &[(0, 1), (1, 2)], vec![(0, region())]);
    assert_eq!(graph.all_regions, vec![region()]);
    #[rustfmt::skip]
    assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::NegativeVolume, None, None, 1)]);
}

#[test]
fn task22j_changed_modifier_creates_bridge_angle_child() {
    let parent = normalized_region();
    let mut child = parent.clone();
    child.bridge_angle = OrcaFloat(37.0);
    let volumes = vec![
        volume(1, ProjectVolumeType::ModelPart, 0.0, 10.0, None),
        volume(
            2,
            ProjectVolumeType::ParameterModifier,
            2.0,
            8.0,
            Some(37.0),
        ),
    ];
    let graph = graph(volumes, &[(0, 1), (1, 2)], vec![(0, parent.clone())]);
    assert_eq!(graph.all_regions, vec![parent, child]);
    #[rustfmt::skip]
    assert_eq!(facts(&graph), vec![
        (0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0),
        (1, 2, ProjectVolumeType::ParameterModifier, Some(0), Some(1), 1),
    ]);
}

#[test]
#[rustfmt::skip]
fn task22j_two_parent_changed_modifier_records_reverse_parent_order() {
    let a = normalized_region();
    let mut b = a.clone(); b.bridge_angle = OrcaFloat(11.0);
    let mut child = a.clone(); child.bridge_angle = OrcaFloat(37.0);
    let volumes = vec![volume(1, ProjectVolumeType::ModelPart, 0.0, 10.0, None), volume(2, ProjectVolumeType::ModelPart, 0.0, 10.0, None), volume(3, ProjectVolumeType::ParameterModifier, 2.0, 8.0, Some(37.0))];
    let graph = graph(volumes, &[(0, 1), (1, 2), (2, 3)], vec![(0, a.clone()), (1, b.clone())]);
    assert_eq!(graph.all_regions, vec![a, b, child]);
    #[rustfmt::skip]
    assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::ModelPart, None, Some(1), 1), (2, 3, ProjectVolumeType::ParameterModifier, Some(1), Some(2), 2), (2, 3, ProjectVolumeType::ParameterModifier, Some(0), Some(2), 2)]);
}

#[test]
#[rustfmt::skip]
fn task22j_modifier_parent_bbox_extends_to_top_model_ancestor() {
    let base = normalized_region(); let mut first = base.clone(); first.bridge_angle = OrcaFloat(11.0); let mut second = base.clone(); second.bridge_angle = OrcaFloat(37.0);
    let volumes = vec![volume(1, ProjectVolumeType::ModelPart, 0.0, 4.0, None), volume(2, ProjectVolumeType::ParameterModifier, 3.0, 6.0, Some(11.0)), volume(3, ProjectVolumeType::ParameterModifier, 0.0, 1.0, Some(37.0))];
    let graph = graph(volumes, &[(0, 1), (1, 2), (2, 3)], vec![(0, base.clone())]);
    assert_eq!(graph.all_regions, vec![base, first, second]);
    assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::ParameterModifier, Some(0), Some(1), 1), (2, 3, ProjectVolumeType::ParameterModifier, Some(1), Some(2), 2), (2, 3, ProjectVolumeType::ParameterModifier, Some(0), Some(2), 2)]);
}

#[test]
#[rustfmt::skip]
fn task22j_noop_modifier_falls_back_only_to_last_intersecting_model_part() {
    let base = normalized_region();
    let volumes = vec![volume(1, ProjectVolumeType::ModelPart, 0.0, 10.0, None), volume(2, ProjectVolumeType::ModelPart, 0.0, 10.0, None), volume(3, ProjectVolumeType::ParameterModifier, 2.0, 8.0, None)];
    let graph = graph(volumes, &[(0, 1), (1, 2), (2, 3)], vec![(0, base.clone()), (1, base.clone())]);
    assert_eq!(graph.all_regions, vec![base]);
    assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::ModelPart, None, Some(0), 1), (2, 3, ProjectVolumeType::ParameterModifier, Some(1), Some(0), 2)]);
}

#[test]
#[rustfmt::skip]
fn task22j_changed_parent_suppresses_remembered_unchanged_fallback() {
    let base = normalized_region(); let mut changed = base.clone(); changed.bridge_angle = OrcaFloat(37.0);
    let volumes = vec![volume(1, ProjectVolumeType::ModelPart, 0.0, 10.0, None), volume(2, ProjectVolumeType::ModelPart, 0.0, 10.0, None), volume(3, ProjectVolumeType::ParameterModifier, 2.0, 8.0, Some(37.0))];
    let graph = graph(volumes, &[(0, 1), (1, 2), (2, 3)], vec![(0, base.clone()), (1, changed.clone())]);
    assert_eq!(graph.all_regions, vec![base, changed]);
    assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::ModelPart, None, Some(1), 1), (2, 3, ProjectVolumeType::ParameterModifier, Some(0), Some(1), 2)]);
}

#[test]
fn task22j_modifier_without_model_ancestor_adds_no_record() {
    let graph = graph(
        vec![volume(
            7,
            ProjectVolumeType::ParameterModifier,
            0.0,
            1.0,
            Some(37.0),
        )],
        &[(0, 9)],
        Vec::new(),
    );
    assert_eq!(graph.all_regions, Vec::<RegionOptions>::new());
    assert_eq!(facts(&graph), Vec::<RecordFact>::new());
}

#[test]
#[rustfmt::skip]
fn task22j_modifier_parent_intersection_is_three_dimensional_and_inclusive() {
    let base = normalized_region();
    let graph_at = |modifier_min_z| graph(vec![volume_at_z(1, ProjectVolumeType::ModelPart, [0.0, 10.0], [0.0, 1.0], None), volume_at_z(2, ProjectVolumeType::ParameterModifier, [0.0, 10.0], [modifier_min_z, 2.0], Some(37.0))], &[(0, 1), (1, 2)], vec![(0, base.clone())]);
    assert_eq!(facts(&graph_at(1.0002)), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::ParameterModifier, Some(0), Some(1), 1)]);
    assert_eq!(facts(&graph_at(1.0003)), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0)]);
}

#[test]
fn task22j_loaded_modifier_and_control_build_exact_region_graphs() {
    let (modifier, control) = modifier_projects();
    for (bytes, changed) in [(&modifier, true), (&control, false)] {
        let (graph, parent, _) = loaded_graph(bytes);
        let mut child = parent.clone();
        child.bridge_angle = OrcaFloat(37.0);
        assert_eq!(
            graph.all_regions,
            if changed {
                vec![parent, child]
            } else {
                vec![parent]
            }
        );
        #[rustfmt::skip]
        assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::ParameterModifier, Some(0), Some(usize::from(changed)), 1)]);
    }
}

#[test]
#[rustfmt::skip]
fn task22j_loaded_bfs_raw_ids_and_support_gap_keep_released_occurrences() {
    let (graph, parent, raw_ids) = loaded_graph(&order_archive(["normal_part", "normal_part", "normal_part"]));
    assert_eq!(raw_ids, vec![3, 1, 2]); assert_eq!(graph.all_regions, vec![parent]);
    assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (1, 2, ProjectVolumeType::ModelPart, None, Some(0), 1), (2, 3, ProjectVolumeType::ModelPart, None, Some(0), 2)]);
    let (graph, parent, raw_ids) = loaded_graph(&order_archive(["normal_part", "support_enforcer", "modifier_part"]));
    assert_eq!(raw_ids, vec![3, 1, 2]); assert_eq!(graph.all_regions, vec![parent]);
    assert_eq!(facts(&graph), vec![(0, 1, ProjectVolumeType::ModelPart, None, Some(0), 0), (2, 3, ProjectVolumeType::ParameterModifier, Some(0), Some(0), 1)]);
}

#[test]
#[rustfmt::skip]
fn task22j_empty_range_document_is_absent_and_real_range_stays_rejected() {
    let absent = KsrArchive::new().bytes(); let mut empty = KsrArchive::new(); empty.insert_text("Metadata/layer_config_ranges.xml", "<objects/>"); let empty = empty.bytes();
    assert_eq!(prepare_project_slice(&absent, None).unwrap().resolved, prepare_project_slice(&empty, None).unwrap().resolved);
    assert_eq!(loaded_graph(&absent).0, loaded_graph(&empty).0);
    let mut ranged = KsrArchive::new(); ranged.insert_text("Metadata/layer_config_ranges.xml", r#"<objects><object id="1"><range min_z="0" max_z="1"><option opt_key="extruder">1</option></range></object></objects>"#);
    assert_eq!(prepare_project_slice(ranged.bytes(), None).err().unwrap(), SliceError::UnsupportedProjectFeature("layer_config_ranges".to_owned()));
}

#[test]
#[rustfmt::skip]
fn task22j_model_part_lookup_scales_on_sorted_source_indices() {
    const COUNT: usize = 4_096;
    let candidates = (0..COUNT).map(|volume_index| ResolvedModelPartCandidate { volume_index, region: region() }).collect::<Vec<_>>();
    assert!((0..COUNT).all(|source| model_part_for_source_index(&candidates, source).volume_index == source));
}

fn graph(
    volumes: Vec<ProjectVolume>,
    occurrences: &[(usize, u32)],
    regions: Vec<(usize, RegionOptions)>,
) -> VolumeRegionGraph {
    let source = object("regions.model", 1, volumes, &[Transform3d::IDENTITY]);
    let mut resolved = resolved_object(0, &[Transform3d::IDENTITY]);
    resolved.layer_candidates[0].model_parts = regions
        .into_iter()
        .map(|(volume_index, region)| ResolvedModelPartCandidate {
            volume_index,
            region,
        })
        .collect();
    let bounded = build_volume_bounds(
        &source,
        &resolved,
        PostClosingPrintObject::new(
            plan(0, 0, 1),
            occurrences
                .iter()
                .map(|&(source_index, ordinal)| {
                    PostClosingVolume::new(
                        source_index,
                        ordinal,
                        source.volumes()[source_index].volume_type(),
                        vec![PostClosingLayer::new(SlicingMode::Regular, Vec::new())],
                    )
                })
                .collect(),
        ),
    );
    build_volume_region_graph(
        &source,
        &resolved,
        &bounded,
        &FilamentRegionSourceOptions::default(),
        1,
    )
}

fn loaded_graph(bytes: &[u8]) -> (VolumeRegionGraph, RegionOptions, Vec<u32>) {
    let PreparedPostClosing {
        project,
        resolved,
        objects,
        ..
    } = prepare_post_simplification(ProjectSource::from(bytes)).unwrap();
    let source = &project.objects()[0];
    let resolved_object = &resolved.objects[0];
    let parent = resolved_object.layer_candidates[0].model_parts[0]
        .region
        .clone();
    let bounded = build_volume_bounds(source, resolved_object, objects.into_iter().next().unwrap());
    let graph = build_volume_region_graph(
        source,
        resolved_object,
        &bounded,
        &resolved.views.full.filament.region,
        resolved.logical_filament_count,
    );
    (
        graph,
        parent,
        source.volumes().iter().map(ProjectVolume::id).collect(),
    )
}

fn facts(graph: &VolumeRegionGraph) -> Vec<RecordFact> {
    graph
        .volume_regions
        .iter()
        .map(|record| {
            (
                record.source_volume_index,
                record.occurrence_id.get(),
                record.kind,
                record.parent,
                record.region_id,
                record.bound_index,
            )
        })
        .collect()
}

fn volume(
    id: u32,
    kind: ProjectVolumeType,
    min_x: f64,
    max_x: f64,
    bridge_angle: Option<f64>,
) -> ProjectVolume {
    volume_at_z(id, kind, [min_x, max_x], [0.0, 1.0], bridge_angle)
}

fn volume_at_z(
    id: u32,
    kind: ProjectVolumeType,
    x: [f64; 2],
    z: [f64; 2],
    bridge_angle: Option<f64>,
) -> ProjectVolume {
    ProjectVolume::new(
        "regions.model".to_owned(),
        id,
        ProjectMesh::new(
            vec![
                Point3d::new(x[0], 0.0, z[0]),
                Point3d::new(x[1], 0.0, z[0]),
                Point3d::new(x[0], 1.0, z[1]),
            ],
            vec![[0, 1, 2]],
        ),
        Transform3d::IDENTITY,
        (
            format!("volume-{id}"),
            kind,
            RegionOptionOverrides {
                bridge_angle: bridge_angle.map(OrcaFloat),
                ..Default::default()
            },
            Transform3d::IDENTITY,
        ),
    )
}

fn normalized_region() -> RegionOptions {
    let mut value = region();
    for field in [
        &mut value.sparse_infill_filament_id,
        &mut value.internal_solid_filament_id,
        &mut value.top_surface_filament_id,
        &mut value.bottom_surface_filament_id,
        &mut value.outer_wall_filament_id,
        &mut value.inner_wall_filament_id,
    ] {
        *field = OrcaInt(1);
    }
    value
}

const ORDER_MESH: &str = r#"<mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="0" y="10" z="0"/><vertex x="0" y="0" z="10"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="1" v3="3"/><triangle v1="0" v2="3" v3="2"/><triangle v1="1" v2="2" v3="3"/></triangles></mesh>"#;

#[rustfmt::skip]
fn order_archive(subtypes: [&str; 3]) -> Vec<u8> {
    let ids = [3, 1, 2]; let path = "/3D/Objects/task22j_order.model";
    let components = ids.iter().map(|id| format!(r#"<component p:path="{path}" objectid="{id}"/>"#)).collect::<String>();
    let objects = ids.iter().map(|id| format!(r#"<object id="{id}" type="model">{ORDER_MESH}</object>"#)).collect::<String>();
    let parts = ids.into_iter().zip(subtypes).map(|(id, subtype)| format!(r#"<part id="{id}" subtype="{subtype}"/>"#)).collect::<String>();
    let mut archive = KsrArchive::new();
    archive.insert_text("3D/3dmodel.model", &format!(r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="10" type="model"><components>{components}</components></object></resources><build><item objectid="10" printable="1"/></build></model>"#));
    archive.insert_text("3D/_rels/3dmodel.model.rels", r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22j_order.model" Id="order" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#);
    archive.insert_text("3D/Objects/task22j_order.model", &format!(r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>{objects}</resources><build/></model>"#));
    archive.insert_text("Metadata/model_settings.config", &format!(r#"<config><object id="10">{parts}</object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="10"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22002"/></model_instance></plate><assemble><assemble_item object_id="10" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#));
    archive.bytes()
}
