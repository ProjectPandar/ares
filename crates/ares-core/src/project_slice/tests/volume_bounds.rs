use crate::{
    OrcaFloat, Point3d, ProjectMesh, ProjectVolume, ProjectVolumeType, Transform3d,
    mesh_slicer::SlicingMode, options::RegionOptionOverrides,
};

use super::{
    super::{
        closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
        volume_bounds::{BoundingBox3f, build_volume_bounds},
    },
    support::{object, plan, project_volume, resolved_object},
};

#[test]
fn task22j_volume_bounds_promote_ordinals_once_and_preserve_carrier_order() {
    let source = object(
        "synthetic.model",
        1,
        vec![
            project_volume(
                "synthetic.model",
                3,
                ProjectVolumeType::ModelPart,
                true,
                false,
            ),
            project_volume(
                "synthetic.model",
                1,
                ProjectVolumeType::ParameterModifier,
                true,
                false,
            ),
        ],
        &[Transform3d::IDENTITY],
    );
    let post_i = PostClosingPrintObject::new(
        plan(0, 0, 1),
        vec![
            post_volume(1, 90, ProjectVolumeType::ParameterModifier),
            post_volume(0, 10, ProjectVolumeType::ModelPart),
        ],
    );
    let bounded = build_volume_bounds(
        &source,
        &resolved_object(0, &[Transform3d::IDENTITY]),
        post_i,
    );

    let source_zero = bounded.volume_for_source_index(0).unwrap();
    assert_eq!(
        (
            source_zero.source_volume_index(),
            source_zero.occurrence_id().get(),
            source_zero.volume_type()
        ),
        (0, 10, ProjectVolumeType::ModelPart)
    );
    assert_eq!(
        bounded
            .bound_for_source_index(1)
            .unwrap()
            .occurrence_id()
            .get(),
        90
    );
    assert_eq!(
        bounded
            .bound_for_occurrence(source_zero.occurrence_id())
            .unwrap()
            .source_volume_index(),
        0
    );
    let (planned, volumes, bounds) = bounded.into_parts();
    assert_eq!(
        (planned.source_object_index, planned.transform_index),
        (0, 0)
    );
    assert_eq!(carrier_facts(volumes), vec![(1, 90, 2, 1), (0, 10, 0, 1)]);
    assert_eq!(bound_occurrences(&bounds), vec![10, 90]);

    let source = object(
        "synthetic.model",
        2,
        [
            ProjectVolumeType::ModelPart,
            ProjectVolumeType::SupportEnforcer,
            ProjectVolumeType::ParameterModifier,
        ]
        .map(|kind| project_volume("synthetic.model", 1, kind, true, false))
        .to_vec(),
        &[Transform3d::IDENTITY],
    );
    let post_i = PostClosingPrintObject::new(
        plan(0, 0, 1),
        vec![
            post_volume(0, 1, ProjectVolumeType::ModelPart),
            post_volume(2, 3, ProjectVolumeType::ParameterModifier),
        ],
    );
    let bounded = build_volume_bounds(
        &source,
        &resolved_object(0, &[Transform3d::IDENTITY]),
        post_i,
    );
    assert_eq!(
        [0, 1, 2].map(|source| bounded.volume_position_for_source_index(source)),
        [Some(0), None, Some(1)]
    );
    let (_, volumes, bounds) = bounded.into_parts();
    assert_eq!(carrier_facts(volumes), vec![(0, 1, 0, 1), (2, 3, 2, 1)]);
    assert_eq!(bound_occurrences(&bounds), vec![1, 3]);
}

#[test]
fn task22j_volume_bounds_apply_transform_compensation_and_epsilon_exactly() {
    let object_transform = row_major([
        2.0, 0.0, 0.0, 100.0, 0.0, 3.0, 0.0, -50.0, 0.0, 0.0, 4.0, 7.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let volume_transform = row_major([
        1.0, 0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 20.0, 0.0, 0.0, 1.0, 5.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let vertices = vec![
        Point3d::new(1.0, 2.0, 3.0),
        Point3d::new(-1.0, 4.0, 2.0),
        Point3d::new(3.0, -2.0, 1.0),
        Point3d::new(1e9, -1e9, 1e9),
    ];
    let positive = bbox(object_transform, volume_transform, vertices.clone(), 2.0);
    assert_bits(positive.min(), [0xc080_0000, 0xc100_0000, 0x41f7_ffcc]);
    assert_bits(positive.max(), [0x4100_0000, 0x4160_0000, 0x421c_001a]);

    for compensation in [0.0, -2.0] {
        let clamped = bbox(
            object_transform,
            volume_transform,
            vertices.clone(),
            compensation,
        );
        assert_bits(clamped.min(), [0xc000_0000, 0xc0c0_0000, 0x41f7_ffcc]);
        assert_bits(clamped.max(), [0x40c0_0000, 0x4140_0000, 0x421c_001a]);
    }
}

#[test]
fn task22j_volume_bounds_compose_f64_then_narrow_matrix_and_vertices_to_f32() {
    let object_transform = row_major([
        1.0000000596046448,
        0.3333333432674408,
        -0.1428571492433548,
        10000000.125,
        0.222_222_238_779_068,
        -0.9999999403953552,
        0.0909090973436832,
        -20000000.375,
        0.1250000074505806,
        0.2857142984867096,
        1.0000001192092896,
        std::f64::consts::PI,
        0.0,
        0.0,
        0.0,
        1.0,
    ]);
    let volume_transform = row_major([
        -0.7777777910232544,
        0.111_111_119_389_534,
        0.2500000298023224,
        5.123456789,
        0.166_666_679_084_301,
        1.0000000596046448,
        -0.2000000029802322,
        -7.987654321,
        0.300_000_011_920_929,
        -0.0833333383002281,
        0.9999999403953552,
        std::f64::consts::E,
        0.0,
        0.0,
        0.0,
        1.0,
    ]);
    let vertices = vec![
        Point3d::new(1.234567890123, -2.345678901234, 0.123456789012),
        Point3d::new(-3.456789012345, 4.567890123456, 2.345678901234),
        Point3d::new(5.678901234567, -6.789012345678, -1.234567890123),
        Point3d::new(1000000000.25, -1000000000.5, 1000000000.75),
    ];
    let fixed = bbox(
        object_transform,
        volume_transform,
        vertices,
        0.3750000298023224,
    );
    assert_bits(fixed.min(), [0xc0fb_c62e, 0xc03e_4085, 0x403e_0660]);
    assert_bits(fixed.max(), [0x40a6_5ffa, 0x409b_1a1c, 0x40d4_0405]);

    let scale = row_major([
        1.00000006, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let point = Point3d::new(1.00000006, 0.0, 1.0);
    let discriminator = bbox(scale, scale, vec![point, point, point], 0.0);
    assert_eq!(discriminator.min()[0].to_bits(), 0x3f80_0002);
    assert_eq!(discriminator.max()[0].to_bits(), 0x3f80_0002);
}

#[test]
fn task22j_volume_bounds_use_inclusive_xy_z_queries_and_extend_ancestors() {
    let base = identity_bbox([
        Point3d::new(0.0, 0.0, 1.0),
        Point3d::new(1.0, 0.0, 1.0),
        Point3d::new(0.0, 1.0, 1.0),
    ]);
    let touch_x = identity_bbox([
        Point3d::new(1.0, 0.25, 1.0),
        Point3d::new(2.0, 0.25, 1.0),
        Point3d::new(1.0, 0.75, 1.0),
    ]);
    let touch_y = identity_bbox([
        Point3d::new(0.25, 1.0, 1.0),
        Point3d::new(0.75, 1.0, 1.0),
        Point3d::new(0.25, 2.0, 1.0),
    ]);
    let separated_x = f32::from_bits(1.0_f32.to_bits() + 1);
    let separated = identity_bbox([
        Point3d::new(f64::from(separated_x), 0.25, 1.0),
        Point3d::new(2.0, 0.25, 1.0),
        Point3d::new(f64::from(separated_x), 0.75, 1.0),
    ]);

    assert!(base.intersects_xy(touch_x));
    assert!(base.intersects_xy(touch_y));
    assert!(!base.intersects_xy(separated));
    let min_z = base.min()[2];
    let max_z = base.max()[2];
    assert!(base.contains_z(min_z));
    assert!(base.contains_z(max_z));
    assert!(!base.contains_z(f32::from_bits(min_z.to_bits() - 1)));
    assert!(!base.contains_z(f32::from_bits(max_z.to_bits() + 1)));

    let mut ancestor = base;
    ancestor.extend(separated);
    assert_eq!(ancestor.min(), base.min());
    assert_eq!(ancestor.max(), [2.0, 1.0, max_z]);
}

fn bbox(
    object_transform: Transform3d,
    volume_transform: Transform3d,
    vertices: Vec<Point3d>,
    compensation: f64,
) -> BoundingBox3f {
    let volume = source_volume(1, ProjectVolumeType::ModelPart, vertices, volume_transform);
    let source = object("synthetic.model", 1, vec![volume], &[object_transform]);
    let mut resolved = resolved_object(0, &[object_transform]);
    resolved.object.xy_contour_compensation = OrcaFloat(compensation);
    let post_i = PostClosingPrintObject::new(
        plan(0, 0, 1),
        vec![post_volume(0, 1, ProjectVolumeType::ModelPart)],
    );
    build_volume_bounds(&source, &resolved, post_i)
        .bound_for_source_index(0)
        .unwrap()
        .bbox()
}

fn identity_bbox(vertices: [Point3d; 3]) -> BoundingBox3f {
    bbox(
        Transform3d::IDENTITY,
        Transform3d::IDENTITY,
        vertices.to_vec(),
        0.0,
    )
}

fn source_volume(
    id: u32,
    kind: ProjectVolumeType,
    vertices: Vec<Point3d>,
    transform: Transform3d,
) -> ProjectVolume {
    ProjectVolume::new(
        "synthetic.model".to_owned(),
        id,
        ProjectMesh::new(vertices, vec![[0, 1, 2]]),
        transform,
        (
            format!("volume-{id}"),
            kind,
            RegionOptionOverrides::default(),
            Transform3d::IDENTITY,
        ),
    )
}

fn post_volume(
    source_volume_index: usize,
    ordinal: u32,
    volume_type: ProjectVolumeType,
) -> PostClosingVolume {
    PostClosingVolume::new(
        source_volume_index,
        ordinal,
        volume_type,
        vec![PostClosingLayer::new(SlicingMode::EvenOdd, Vec::new())],
    )
}

fn carrier_facts(
    volumes: Vec<super::super::volume_bounds::PostBoundsVolume>,
) -> Vec<(usize, u32, u8, usize)> {
    volumes
        .into_iter()
        .map(|volume| {
            let (source, occurrence, kind, layers) = volume.into_parts();
            let kind = match kind {
                ProjectVolumeType::ModelPart => 0,
                ProjectVolumeType::ParameterModifier => 2,
                _ => unreachable!(),
            };
            assert!(
                layers
                    .iter()
                    .all(|layer| layer.mode() == SlicingMode::EvenOdd)
            );
            (source, occurrence.get(), kind, layers.len())
        })
        .collect()
}

fn bound_occurrences(bounds: &[super::super::volume_bounds::VolumeBound]) -> Vec<u32> {
    bounds
        .iter()
        .map(|bound| {
            let (source, occurrence, bbox) = bound.into_parts();
            assert_eq!(source, bound.source_volume_index());
            assert_eq!(bbox, bound.bbox());
            occurrence.get()
        })
        .collect()
}

fn assert_bits(actual: [f32; 3], expected: [u32; 3]) {
    assert_eq!(actual.map(f32::to_bits), expected);
}

fn row_major(values: [f64; 16]) -> Transform3d {
    Transform3d::parse_row_major(
        &values
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(" "),
    )
    .unwrap()
}
