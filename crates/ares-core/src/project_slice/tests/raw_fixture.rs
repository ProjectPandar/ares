mod closed_components;
mod encoding;
mod mutations;

use sha2::{Digest, Sha256};

use crate::{
    Point3d, ProjectMesh, ProjectVolumeType, SliceError, Transform3d,
    geometry::CoordinateScale,
    mesh_slicer::{IntersectionLine, index_mesh_edges},
    slice_project,
};

use super::{
    super::state::prepare_project_slice,
    support::{ksr_project, metadata},
};
use encoding::{
    EDGE, GENERAL, LineRecord, ObjectView, TOP, VERTEX, VolumeView, encode, line_record,
    sorted_records,
};

const PROJECT_SHA256: &str = "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";
const CONFIG_BLOCK_SHA256: &str =
    "b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8";
const SEMANTIC_SHA256: &str = "a82b2d193c23c8ba499c7abd56e21cb9956f5444e9b51b1b261a7e9b67d26d21";
const FACE_ORDER_SHA256: &str = "1a6e83f2d5f53b73fa7ba9cb6444909816276496361f7fb9f9305412d2045e79";

#[tokio::test]
async fn task22b_ksr_fixture_matches_exact_raw_counts_components_records_and_digests() {
    let project_bytes = ksr_project();
    assert_eq!(project_bytes.len(), 183_007);
    assert_eq!(sha256(project_bytes), PROJECT_SHA256);

    {
        let state = prepare_project_slice(project_bytes).unwrap();
        assert_eq!(state.project.objects().len(), 1);
        let source_object = &state.project.objects()[0];
        assert!(source_object.layer_config_ranges().is_empty());
        assert_eq!(source_object.volumes().len(), 1);
        let source_volume = &source_object.volumes()[0];
        assert_eq!(source_volume.volume_type(), ProjectVolumeType::ModelPart);
        assert_import_oracles(source_volume.mesh());
        assert_eq!(source_volume.transform(), Transform3d::IDENTITY);
        assert_eq!(
            2 + source_volume.mesh().vertices().len() + source_volume.mesh().triangles().len(),
            18_345
        );
        assert_topology_oracles(source_volume.mesh());

        assert_eq!(state.resolved.logical_filament_count, 2);
        assert_eq!(state.resolved.objects.len(), 1);
        let candidates = &state.resolved.objects[0].layer_candidates;
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            (
                candidates[0].min_z,
                candidates[0].max_z,
                candidates[0].source_range_index,
            ),
            (0.0, f64::MAX, None)
        );
        assert_eq!(candidates[0].model_parts.len(), 1);
        assert_eq!(candidates[0].model_parts[0].volume_index, 0);
        let full = &state.resolved.views.full;
        assert_eq!(
            full.filament
                .print
                .filament_shrink
                .0
                .iter()
                .map(|value| value.0)
                .collect::<Vec<_>>(),
            [100.0, 100.0]
        );
        assert_eq!(
            full.filament
                .print
                .filament_shrinkage_compensation_z
                .0
                .iter()
                .map(|value| value.0)
                .collect::<Vec<_>>(),
            [100.0, 100.0]
        );
        assert_eq!(
            CoordinateScale::from_printable_area(&full.printer.remaining.printable_area).factor(),
            0.000_001
        );
        let config_block = state.config_block.as_deref().unwrap();
        assert_eq!(config_block.len(), 49_004);
        assert_eq!(sha256(config_block), CONFIG_BLOCK_SHA256);

        assert_eq!(state.intersected_objects.len(), 1);
        let object = &state.intersected_objects[0];
        assert_eq!(object.plan.source_object_index, 0);
        assert_eq!(object.plan.transform_index, 0);
        assert_eq!(object.plan.layers.len(), 460);
        assert_eq!(object.volumes().len(), 1);
        let volume = &object.volumes()[0];
        assert_eq!(volume.ordinal(), 1);
        assert_eq!(volume.volume_type(), ProjectVolumeType::ModelPart);
        let layers = volume.layers();
        assert_eq!(layers.len(), object.plan.layers.len());
        assert_eq!(object.plan.layers.len() * object.volumes().len(), 460);
        assert_layer_oracles(&object.plan.layers, layers);

        let views = state
            .intersected_objects
            .iter()
            .map(|object| ObjectView {
                source_object_index: object.plan.source_object_index,
                transform_index: object.plan.transform_index,
                volumes: object
                    .volumes()
                    .iter()
                    .map(|volume| VolumeView {
                        ordinal: volume.ordinal(),
                        volume_type: volume.volume_type(),
                        layers: volume.layers(),
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        let semantic = encode(&views, true);
        let face_order = encode(&views, false);
        assert_eq!(semantic.len(), 5_012_035);
        assert_eq!(face_order.len(), 5_012_035);
        assert_eq!(sha256(&semantic), SEMANTIC_SHA256);
        assert_eq!(sha256(&face_order), FACE_ORDER_SHA256);
    }

    assert_eq!(
        slice_project(project_bytes, metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

fn assert_import_oracles(mesh: &ProjectMesh) {
    assert_eq!(mesh.vertices().len(), 6_109);
    assert_eq!(mesh.triangles().len(), 12_234);
    assert_eq!(mesh.triangles()[0], [2, 0, 1]);
    assert_eq!(
        f32_bounds(mesh),
        [[-37.5, -35.0, -46.0], [37.5, 35.0, 46.0]]
    );
    assert!((51_000.0..51_020.0).contains(&signed_volume(mesh)));
}

fn assert_topology_oracles(mesh: &ProjectMesh) {
    let topology = index_mesh_edges(mesh.triangles()).unwrap();
    assert_eq!(topology.edge_count(), 18_351);
    let mut uses = vec![Vec::<(u32, u32)>::new(); topology.edge_count() as usize];
    for (triangle, edge_ids) in mesh.triangles().iter().zip(topology.face_edge_ids()) {
        for local_edge in 0..3 {
            uses[edge_ids[local_edge] as usize]
                .push((triangle[local_edge], triangle[(local_edge + 1) % 3]));
        }
    }
    for edge_uses in uses {
        assert_eq!(edge_uses.len(), 2);
        assert_eq!(edge_uses[0], (edge_uses[1].1, edge_uses[1].0));
    }
}

fn assert_layer_oracles(
    planned_layers: &[super::super::layers::PlannedLayer],
    layers: &[Vec<IntersectionLine>],
) {
    let total_lines = layers.iter().map(Vec::len).sum::<usize>();
    assert_eq!(total_lines, 116_472);
    assert_eq!(
        layers
            .iter()
            .enumerate()
            .max_by_key(|(_, lines)| lines.len())
            .map(|(index, lines)| (index, lines.len())),
        Some((46, 3_011))
    );
    assert_eq!(
        f64::from(planned_layers[46].slice_z as f32),
        9.300_000_190_734_863
    );
    assert_eq!(closed_components::count(&layers[46]), 41);
    for (layer, line_count, component_count) in [
        (0, 1_046, 12),
        (2, 932, 12),
        (12, 1_265, 12),
        (17, 1_138, 12),
        (37, 880, 15),
        (230, 38, 1),
        (459, 72, 9),
    ] {
        assert_eq!(layers[layer].len(), line_count, "layer {layer}");
        assert_eq!(
            closed_components::count(&layers[layer]),
            component_count,
            "layer {layer}"
        );
    }
    assert!(
        layers
            .iter()
            .flatten()
            .all(|line| line.a().point() != line.b().point())
    );
    assert_representative_records(layers);
}

fn assert_representative_records(layers: &[Vec<IntersectionLine>]) {
    let layer_0_face_order = LineRecord::new(
        (17_530_508, -25_999_317, EDGE, 0),
        (17_983_121, -25_954_736, EDGE, 1),
        GENERAL,
    );
    let layer_0_sorted = LineRecord::new(
        (-37_500_000, -33_000_000, EDGE, 6_691),
        (-37_469_924, -33_343_825, EDGE, 6_982),
        GENERAL,
    );
    let layer_2_top = LineRecord::new(
        (17_043_610, -26_369_232, VERTEX, 4),
        (17_652_542, -26_396_576, VERTEX, 0),
        TOP,
    );
    let layer_37_top = LineRecord::new(
        (17_043_610, -26_369_232, VERTEX, 11),
        (17_652_542, -26_396_576, VERTEX, 5),
        TOP,
    );
    let layer_459_sorted = LineRecord::new(
        (2_196_466, -30_303_541, EDGE, 11_738),
        (2_201_466, -30_303_541, EDGE, 11_741),
        GENERAL,
    );

    assert_eq!(line_record(layers[0][0]), layer_0_face_order);
    assert_eq!(sorted_records(&layers[0])[0], layer_0_sorted);
    assert_eq!(sorted_records(&layers[459])[0], layer_459_sorted);
    assert_eq!(
        layers[2]
            .iter()
            .copied()
            .filter(|line| line_record(*line) == layer_2_top)
            .count(),
        1
    );
    assert_eq!(
        layers[37]
            .iter()
            .copied()
            .filter(|line| line_record(*line) == layer_37_top)
            .count(),
        1
    );
}

fn f32_bounds(mesh: &ProjectMesh) -> [[f32; 3]; 2] {
    mesh.vertices().iter().fold(
        [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]],
        |mut bounds, point| {
            for (axis, value) in [point.x as f32, point.y as f32, point.z as f32]
                .into_iter()
                .enumerate()
            {
                bounds[0][axis] = bounds[0][axis].min(value);
                bounds[1][axis] = bounds[1][axis].max(value);
            }
            bounds
        },
    )
}

fn signed_volume(mesh: &ProjectMesh) -> f32 {
    let vertices = mesh.vertices();
    let origin = point_f32(vertices[0]);
    mesh.triangles().iter().fold(0.0, |volume, triangle| {
        let [a, b, c] = triangle.map(|index| point_f32(vertices[index as usize]));
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let norm = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let normal = if norm == 0.0 {
            cross
        } else {
            cross.map(|component| component / norm)
        };
        let height = normal[0] * (a[0] - origin[0])
            + normal[1] * (a[1] - origin[1])
            + normal[2] * (a[2] - origin[2]);
        volume + 0.5 * norm * height / 3.0
    })
}

fn point_f32(point: Point3d) -> [f32; 3] {
    [point.x as f32, point.y as f32, point.z as f32]
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
