use crate::{
    Point3d, ProjectVolume, ProjectVolumeType, Transform3d,
    geometry::{CoordinateScale, Point},
};

use super::{
    super::{
        chained_intersections::chain_project_intersections,
        looped_intersections::loop_project_intersections, state::prepare_project_slice,
    },
    raw_support::{intersections, mesh_volume, planned_layers},
    support::{KsrArchive, identity_resolved, ksr_project, object},
};

const NORMAL_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"256x0\",\r\n",
    "\t\t\"256x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);
const LARGE_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"2148x0\",\r\n",
    "\t\t\"2148x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);

#[test]
fn task22d_project_state_retains_request_scale_and_source_gap_radius() {
    let normal = prepare_project_slice(ksr_project()).unwrap();
    assert_eq!(normal.scale, CoordinateScale::Normal);
    assert_eq!(normal.scale.checked_scale(2.0), Some(2_000_000));

    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        NORMAL_PRINTABLE_AREA,
        LARGE_PRINTABLE_AREA,
    );
    let large = prepare_project_slice(archive.bytes()).unwrap();
    assert_eq!(large.scale, CoordinateScale::LargeBed);
    assert_eq!(large.scale.checked_scale(2.0), Some(199_999));
}

#[test]
fn task22d_looped_wrapper_preserves_project_ownership_and_polygon_order() {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier};

    let source_objects = vec![
        object(
            "looped.model",
            10,
            vec![
                tetra_volume(10, ModelPart, &[0.0]),
                tetra_volume(11, ParameterModifier, &[0.0, 4.0]),
            ],
            &[Transform3d::IDENTITY],
        ),
        object(
            "looped.model",
            20,
            vec![
                tetra_volume(20, ModelPart, &[0.0]),
                tetra_volume(21, NegativeVolume, &[0.0]),
            ],
            &[Transform3d::IDENTITY],
        ),
    ];
    let plans = vec![
        planned_layers(0, 0, &[(100.0, 0.5), (101.0, 3.5)]),
        planned_layers(1, 0, &[(200.0, 0.5), (201.0, 3.5)]),
    ];
    let raw = intersections(
        &source_objects,
        &[identity_resolved(0), identity_resolved(1)],
        plans.clone(),
    )
    .unwrap();
    let chained = chain_project_intersections(raw);
    let looped = loop_project_intersections(chained, 2_000_000);

    assert_eq!(looped.len(), 2);
    for (object, plan) in looped.iter().zip(&plans) {
        assert_eq!(object.plan(), plan);
    }
    assert_eq!(
        volume_metadata(&looped[0]),
        [(1, ModelPart), (2, ParameterModifier)]
    );
    assert_eq!(
        volume_metadata(&looped[1]),
        [(1, ModelPart), (2, NegativeVolume)]
    );

    let expected = [
        Point::new(-1_000_000, -1_000_000),
        Point::new(500_000, -1_000_000),
        Point::new(-1_000_000, 500_000),
    ];
    for volume in looped.iter().flat_map(|object| object.volumes()) {
        assert_eq!(volume.layers().len(), 2);
        assert!(volume.layers()[1].polygons().is_empty());
    }
    for volume in [
        &looped[0].volumes()[0],
        &looped[1].volumes()[0],
        &looped[1].volumes()[1],
    ] {
        assert_eq!(volume.layers()[0].polygons().len(), 1);
        assert_eq!(volume.layers()[0].polygons()[0].points(), expected);
    }
    let ordered = looped[0].volumes()[1].layers()[0].polygons();
    assert_eq!(ordered.len(), 2);
    assert_eq!(ordered[0].points(), expected);
    assert_eq!(
        ordered[1].points(),
        [
            Point::new(3_000_000, -1_000_000),
            Point::new(4_500_000, -1_000_000),
            Point::new(3_000_000, 500_000),
        ]
    );
}

#[test]
fn task22d_looped_wrapper_repairs_a_project_open_polyline() {
    let source_objects = vec![object(
        "repairable-open.model",
        30,
        vec![mesh_volume(
            30,
            ProjectVolumeType::ModelPart,
            vec![
                Point3d::new(0.0, 0.0, 0.0),
                Point3d::new(1.0, 0.0, 0.0),
                Point3d::new(0.0, 1.0, 0.0),
                Point3d::new(0.0, 0.0, 2.0),
            ],
            vec![[0, 1, 3], [1, 2, 3]],
            Transform3d::IDENTITY,
        )],
        &[Transform3d::IDENTITY],
    )];
    let plans = vec![planned_layers(0, 0, &[(100.0, 0.5)])];
    let raw = intersections(&source_objects, &[identity_resolved(0)], plans).unwrap();
    let chained = chain_project_intersections(raw);
    let chained_layer = &chained[0].volumes()[0].layers()[0];
    assert!(chained_layer.polygons().is_empty());
    assert_eq!(chained_layer.open_polylines().len(), 1);
    assert_eq!(chained_layer.open_polylines()[0].points().len(), 3);

    let looped = loop_project_intersections(chained, 2_000_000);

    let polygons = looped[0].volumes()[0].layers()[0].polygons();
    assert_eq!(polygons.len(), 1);
    assert_eq!(
        polygons[0].points(),
        [
            Point::new(-500_000, -500_000),
            Point::new(250_000, -500_000),
            Point::new(-500_000, 250_000),
        ]
    );
}

fn tetra_volume(id: u32, volume_type: ProjectVolumeType, x_offsets: &[f64]) -> ProjectVolume {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for &x in x_offsets {
        let base = u32::try_from(vertices.len()).unwrap();
        vertices.extend([
            Point3d::new(x, 0.0, 0.0),
            Point3d::new(x + 2.0, 0.0, 0.0),
            Point3d::new(x, 2.0, 0.0),
            Point3d::new(x, 0.0, 2.0),
        ]);
        triangles.extend([
            [base, base + 2, base + 1],
            [base, base + 1, base + 3],
            [base, base + 3, base + 2],
            [base + 1, base + 2, base + 3],
        ]);
    }
    mesh_volume(id, volume_type, vertices, triangles, Transform3d::IDENTITY)
}

fn volume_metadata(
    object: &super::super::looped_intersections::LoopedPrintObject,
) -> Vec<(u32, ProjectVolumeType)> {
    object
        .volumes()
        .iter()
        .map(|volume| (volume.ordinal(), volume.volume_type()))
        .collect()
}
