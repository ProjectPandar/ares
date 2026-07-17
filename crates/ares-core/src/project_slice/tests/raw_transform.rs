use crate::{
    Point3d, ProjectVolumeType, Transform3d,
    geometry::CoordinateScale,
    mesh_slicer::{EndpointReference, FacetEdgeType, IntersectionLine},
};

use super::{
    super::raw_intersections::raw_center,
    raw_support::{intersections as intersect, mesh_volume as volume, planned_layers as planned},
    support::{identity_resolved, object, resolved_object, transform},
};

#[test]
fn task22b_raw_center_quantizes_importer_f32_vertices_before_unscale() {
    let extent = f64::from(0.000_004_f32);
    let y_lo = f64::from(-0.000_001_f32);
    let y_hi = f64::from(0.000_001_f32);
    let cases = [
        (0.0, extent, (-1, 0), (1, 0)),
        (-extent, 0.0, (-3, 0), (-1, 0)),
    ];

    for (lo, hi, expected_a, expected_b) in cases {
        let source = object(
            "center.model",
            1,
            vec![volume(
                1,
                ProjectVolumeType::ModelPart,
                vec![
                    Point3d::new(lo, y_lo, 0.0),
                    Point3d::new(hi, y_lo, 0.0),
                    Point3d::new(lo, y_hi, 1.0),
                ],
                vec![[0, 1, 2]],
                Transform3d::IDENTITY,
            )],
            &[Transform3d::IDENTITY],
        );
        assert_eq!(
            raw_center(&source, CoordinateScale::Normal).unwrap().x(),
            if lo == 0.0 { 1 } else { -1 }
        );
        let objects = intersect(
            std::slice::from_ref(&source),
            &[identity_resolved(0)],
            vec![planned(0, 0, &[(7.0, 0.5)])],
        )
        .unwrap();

        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].volumes().len(), 1);
        assert_eq!(objects[0].volumes()[0].ordinal(), 1);
        assert_eq!(objects[0].volumes()[0].layers().len(), 1);
        assert_eq!(objects[0].volumes()[0].layers()[0].len(), 1);
        assert_line(
            objects[0].volumes()[0].layers()[0][0],
            LineSignature {
                a: expected_a,
                a_reference: EndpointReference::Edge(1),
                b: expected_b,
                b_reference: EndpointReference::Edge(2),
                edge_type: FacetEdgeType::General,
            },
        );
    }

    let shift = 128.000_007_629_394_53;
    let delta = 0.000_015_258_789_062_5;
    let source = object(
        "asymmetric-center.model",
        2,
        vec![volume(
            2,
            ProjectVolumeType::ModelPart,
            vec![
                Point3d::new(0.0, -0.000_001, 0.0),
                Point3d::new(delta, -0.000_001, 0.0),
                Point3d::new(0.0, 0.000_001, 1.0),
            ],
            vec![[0, 1, 2]],
            translation_x(shift),
        )],
        &[Transform3d::IDENTITY],
    );
    let scale = CoordinateScale::Normal;
    assert_eq!(raw_center(&source, scale).unwrap().x(), 128_000_015);
    assert_eq!(scale.checked_scale(shift), Some(128_000_007));
}

#[test]
fn task22b_raw_center_uses_f64_transforms_all_vertices_and_model_parts_only() {
    let scaled = object(
        "scaled.model",
        1,
        vec![volume(
            1,
            ProjectVolumeType::ModelPart,
            vec![
                Point3d::new(0.0, -0.000_001, 0.0),
                Point3d::new(4.0, -0.000_001, 0.0),
                Point3d::new(0.0, 0.000_001, 1.0),
            ],
            vec![[0, 1, 2]],
            transform("0.000001 0 0 0 1 0 0 0 1 0 0 0"),
        )],
        &[Transform3d::IDENTITY],
    );
    let scaled = intersect(
        std::slice::from_ref(&scaled),
        &[identity_resolved(0)],
        vec![planned(0, 0, &[(9.0, 0.5)])],
    )
    .unwrap();
    assert_line(
        scaled[0].volumes()[0].layers()[0][0],
        LineSignature {
            a: (-2, 0),
            a_reference: EndpointReference::Edge(1),
            b: (0, 0),
            b_reference: EndpointReference::Edge(2),
            edge_type: FacetEdgeType::General,
        },
    );

    let extent = f64::from(0.000_004_f32);
    let translated = transform("1 0 0 0 1 0 0 0 1 100 200 3");
    let group = transform("1 0 0 0 1 0 0 0 1 0 0 3");
    let source = object(
        "all-vertices.model",
        2,
        vec![
            volume(
                10,
                ProjectVolumeType::ModelPart,
                vec![
                    Point3d::new(0.0, -0.000_001, 0.0),
                    Point3d::new(extent, -0.000_001, 0.0),
                    Point3d::new(0.0, 0.000_001, 1.0),
                    Point3d::new(2.0 * extent, 0.0, 0.0),
                ],
                vec![[0, 1, 2]],
                Transform3d::IDENTITY,
            ),
            volume(
                11,
                ProjectVolumeType::NegativeVolume,
                far_triangle(),
                vec![[0, 1, 2]],
                Transform3d::IDENTITY,
            ),
            volume(
                12,
                ProjectVolumeType::ParameterModifier,
                far_triangle(),
                vec![[0, 1, 2]],
                Transform3d::IDENTITY,
            ),
            volume(
                13,
                ProjectVolumeType::ModelPart,
                vec![Point3d::new(1_000.0, 0.0, 0.0)],
                Vec::new(),
                Transform3d::IDENTITY,
            ),
        ],
        &[translated],
    );
    let objects = intersect(
        std::slice::from_ref(&source),
        &[resolved_object(0, &[group])],
        vec![planned(0, 0, &[(100.0, 3.5)])],
    )
    .unwrap();

    assert_eq!(objects[0].volumes().len(), 3);
    assert_line(
        objects[0].volumes()[0].layers()[0][0],
        LineSignature {
            a: (-3, 0),
            a_reference: EndpointReference::Edge(1),
            b: (-1, 0),
            b_reference: EndpointReference::Edge(2),
            edge_type: FacetEdgeType::General,
        },
    );
}

#[test]
fn task22b_centered_slice_transform_composes_translation_scale_rotation_and_z_exactly() {
    let source_transform = transform("0 1 0 -2 0 0 0 0 1 100 200 5");
    let group_transform = transform("0 1 0 -2 0 0 0 0 1 0 0 5");
    let volume_transform = transform("1 0 0 0 1 0 0 0 2 1 2 3");
    let source = object(
        "composed.model",
        1,
        vec![volume(
            1,
            ProjectVolumeType::ModelPart,
            vec![
                Point3d::new(0.0, 0.0, 0.0),
                Point3d::new(2.0, 0.0, 0.0),
                Point3d::new(0.0, 2.0, 1.0),
            ],
            vec![[0, 1, 2]],
            volume_transform,
        )],
        &[source_transform],
    );
    let objects = intersect(
        std::slice::from_ref(&source),
        &[resolved_object(0, &[group_transform])],
        vec![planned(0, 0, &[(100.0, 9.0)])],
    )
    .unwrap();

    assert_line(
        objects[0].volumes()[0].layers()[0][0],
        LineSignature {
            a: (0, -1_000_000),
            a_reference: EndpointReference::Edge(1),
            b: (0, 0),
            b_reference: EndpointReference::Edge(2),
            edge_type: FacetEdgeType::General,
        },
    );

    let next_up = f64::from(f32::from_bits(1.0_f32.to_bits() + 1));
    let matrix_discriminator = transform("1 0 0 0 1 0 0 0 16777217 0 0 -16777216");
    let cast_source = object(
        "f32-matrix-cast.model",
        2,
        vec![
            volume(
                20,
                ProjectVolumeType::ModelPart,
                vec![
                    Point3d::new(-0.000_001, -0.000_001, 10.0),
                    Point3d::new(0.000_001, -0.000_001, 10.0),
                    Point3d::new(-0.000_001, 0.000_001, 11.0),
                ],
                vec![[0, 1, 2]],
                Transform3d::IDENTITY,
            ),
            volume(
                21,
                ProjectVolumeType::ParameterModifier,
                vec![
                    Point3d::new(-0.000_001, -0.000_001, 1.0),
                    Point3d::new(0.000_001, -0.000_001, 1.0),
                    Point3d::new(-0.000_001, 0.000_001, next_up),
                ],
                vec![[0, 1, 2]],
                matrix_discriminator,
            ),
        ],
        &[Transform3d::IDENTITY],
    );
    let cast_objects = intersect(
        std::slice::from_ref(&cast_source),
        &[identity_resolved(0)],
        vec![planned(0, 0, &[(50.0, 1.0)])],
    )
    .unwrap();

    assert_eq!(cast_objects[0].volumes()[1].ordinal(), 2);
    assert_eq!(cast_objects[0].volumes()[1].layers()[0].len(), 1);
}

#[test]
fn task22b_mirrored_affine_preserves_import_normalized_indices_and_direction() {
    let mirrored = transform("-1 0 0 0 1 0 0 0 1 0 0 0");
    let source = object(
        "mirrored.model",
        1,
        vec![volume(
            1,
            ProjectVolumeType::ModelPart,
            vec![
                Point3d::new(-0.000_001, -0.000_001, 0.0),
                Point3d::new(0.000_001, -0.000_001, 0.0),
                Point3d::new(-0.000_001, 0.000_001, 1.0),
            ],
            vec![[0, 1, 2]],
            mirrored,
        )],
        &[Transform3d::IDENTITY],
    );
    let objects = intersect(
        std::slice::from_ref(&source),
        &[identity_resolved(0)],
        vec![planned(0, 0, &[(10.0, 0.5)])],
    )
    .unwrap();

    assert_line(
        objects[0].volumes()[0].layers()[0][0],
        LineSignature {
            a: (1, 0),
            a_reference: EndpointReference::Edge(1),
            b: (0, 0),
            b_reference: EndpointReference::Edge(2),
            edge_type: FacetEdgeType::General,
        },
    );
}

fn far_triangle() -> Vec<Point3d> {
    vec![
        Point3d::new(0.001, -0.000_001, 3.0),
        Point3d::new(0.001_004, -0.000_001, 3.0),
        Point3d::new(0.001, 0.000_001, 4.0),
    ]
}

fn translation_x(x: f64) -> Transform3d {
    transform(&format!("1 0 0 0 1 0 0 0 1 {x} 0 0"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LineSignature {
    a: (i64, i64),
    a_reference: EndpointReference,
    b: (i64, i64),
    b_reference: EndpointReference,
    edge_type: FacetEdgeType,
}

fn assert_line(line: IntersectionLine, expected: LineSignature) {
    assert_eq!((line.a().point().x(), line.a().point().y()), expected.a);
    assert_eq!(line.a().reference(), expected.a_reference);
    assert_eq!((line.b().point().x(), line.b().point().y()), expected.b);
    assert_eq!(line.b().reference(), expected.b_reference);
    assert_eq!(line.edge_type(), expected.edge_type);
}
