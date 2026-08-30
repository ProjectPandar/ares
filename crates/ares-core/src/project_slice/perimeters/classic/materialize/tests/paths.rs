use crate::{
    SliceError,
    geometry::{CoordinateScale, Point, Polygon},
    project_slice::perimeters::{
        classic::{
            materialize::{
                path::{materialize_ordinary, materialize_overhang_from_lower, scaled_epsilon},
                types::ExtrusionRole,
            },
            traversal::{
                ClassicTraversalRecord, InactiveOverhangReverse, LowerFlowRoute,
                PendingExtrusionRole, PendingLoopRole, PendingPathBranch, TraversalSeed,
            },
        },
        types::Flow,
    },
};

fn normal_scale() -> CoordinateScale {
    CoordinateScale::from_printable_area(&crate::Point2dList(vec![
        crate::Point2d::new(0.0, 0.0),
        crate::Point2d::new(256.0, 256.0),
    ]))
}

fn seed() -> TraversalSeed {
    TraversalSeed {
        polygon: Polygon::new(vec![
            Point::new(10, 20),
            Point::new(30, 20),
            Point::new(30, 40),
        ]),
        depth: 0,
        is_contour: true,
        is_smaller_width_perimeter: false,
        extrusion_role: PendingExtrusionRole::ExternalPerimeter,
        loop_role: PendingLoopRole::Default,
        route: LowerFlowRoute::External,
        width: 0.419_999_96,
        mm3_per_mm: 0.123_456_789,
        children: Vec::new(),
    }
}

fn record() -> ClassicTraversalRecord {
    ClassicTraversalRecord {
        surfaces: Vec::new(),
        layer_id: 2,
        layer_height: 0.123_456_789_012_3,
        slice_z: 0.25,
        fuzzy_skin: crate::perimeters::FuzzySkinConfig::disabled(),
        simplification_tolerance: 0.012,
        overhang_flow: Flow {
            width: 0.51,
            height: 0.27,
            spacing: 0.4,
            nozzle_diameter: 0.4,
            bridge: true,
            mm3_per_mm: 0.765_432_1,
        },
        branch: PendingPathBranch::OverhangClipping {
            detect_overhang_wall: true,
            layer_id: 2,
            raft_layers: 0,
        },
        overhang_reverse: InactiveOverhangReverse {
            configured: false,
            odd_layer: false,
            active: false,
        },
    }
}

#[test]
fn task22o7_ordinary_path_is_exact_closed_polyline_with_fixed_z_and_layer_cast() {
    let layer_height = 0.123_456_789_012_3;
    let paths = materialize_ordinary(&seed(), layer_height);
    assert_eq!(paths.len(), 1);
    let path = &paths[0];
    assert_eq!(path.role, ExtrusionRole::ExternalPerimeter);
    assert_eq!(path.mm3_per_mm.to_bits(), 0.123_456_789_f64.to_bits());
    assert_eq!(path.width.to_bits(), 0.419_999_96_f32.to_bits());
    assert_eq!(path.height.to_bits(), (layer_height as f32).to_bits());
    assert_ne!(layer_height, f64::from(path.height));
    assert_eq!(
        path.polyline
            .points
            .iter()
            .map(|point| (point.x, point.y, point.z))
            .collect::<Vec<_>>(),
        [(10, 20, 0), (30, 20, 0), (30, 40, 0), (10, 20, 0)]
    );
}

#[test]
fn task22o7_overhang_appends_supported_then_remainder_with_distinct_flows_and_heights() {
    let mut seed = seed();
    seed.polygon = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(40, 0),
        Point::new(40, 40),
        Point::new(0, 40),
    ]);
    let lower = [Polygon::new(vec![
        Point::new(-10, -10),
        Point::new(20, -10),
        Point::new(20, 50),
        Point::new(-10, 50),
    ])];
    let record = record();
    let mut bounds = crate::geometry::BoundingBox::from_polygon(&seed.polygon).unwrap();
    bounds.offset(scaled_epsilon(normal_scale()));
    let filtered = crate::geometry::clip_clipper_polygons_with_subject_bbox(&lower, bounds);
    let subject = std::slice::from_ref(&seed.polygon);
    let expected_points = crate::geometry::intersection_pl(subject, &filtered)
        .unwrap()
        .into_iter()
        .chain(crate::geometry::diff_pl(subject, &filtered).unwrap())
        .map(|polyline| {
            polyline
                .points()
                .iter()
                .map(|point| (point.x(), point.y(), 0))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        expected_points,
        [
            vec![(20, 40, 0), (0, 40, 0), (0, 0, 0), (20, 0, 0)],
            vec![(20, 0, 0), (40, 0, 0), (40, 40, 0), (20, 40, 0)],
        ]
    );
    let paths = materialize_overhang_from_lower(&record, &seed, normal_scale(), &lower).unwrap();
    assert_eq!(
        paths
            .iter()
            .map(|path| {
                path.polyline
                    .points
                    .iter()
                    .map(|point| (point.x, point.y, point.z))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        expected_points
    );
    let first_overhang = paths
        .iter()
        .position(|path| path.role == ExtrusionRole::OverhangPerimeter)
        .unwrap();
    assert!(first_overhang > 0);
    assert!(
        paths[..first_overhang]
            .iter()
            .all(|path| path.role == ExtrusionRole::ExternalPerimeter)
    );
    assert!(
        paths[first_overhang..]
            .iter()
            .all(|path| path.role == ExtrusionRole::OverhangPerimeter)
    );
    for path in &paths[..first_overhang] {
        assert_eq!(path.mm3_per_mm.to_bits(), seed.mm3_per_mm.to_bits());
        assert_eq!(path.width.to_bits(), seed.width.to_bits());
        assert_eq!(
            path.height.to_bits(),
            (record.layer_height as f32).to_bits()
        );
    }
    for path in &paths[first_overhang..] {
        assert_eq!(
            path.mm3_per_mm.to_bits(),
            record.overhang_flow.mm3_per_mm.to_bits()
        );
        assert_eq!(path.width.to_bits(), record.overhang_flow.width.to_bits());
        assert_eq!(path.height.to_bits(), record.overhang_flow.height.to_bits());
    }
}

#[test]
fn task22o7_bbox_prefilter_retains_a_polygon_whose_crossing_edge_has_no_inside_vertex() {
    let mut seed = seed();
    seed.polygon = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(40, 0),
        Point::new(40, 40),
        Point::new(0, 40),
    ]);
    let crossing = [Polygon::new(vec![
        Point::new(-200, 20),
        Point::new(20, -200),
        Point::new(240, 20),
        Point::new(20, 240),
    ])];
    let paths =
        materialize_overhang_from_lower(&record(), &seed, normal_scale(), &crossing).unwrap();
    assert!(
        paths
            .iter()
            .any(|path| path.role != ExtrusionRole::OverhangPerimeter)
    );
}

#[test]
fn task22o7_coordinate_failure_is_reported_without_an_ordinary_fallback() {
    let mut seed = seed();
    let high = 0x4000_0000_0000_0000_i64;
    seed.polygon = Polygon::new(vec![
        Point::new(high, 0),
        Point::new(high + 10, 0),
        Point::new(high + 10, 10),
    ]);
    assert_eq!(
        materialize_overhang_from_lower(&record(), &seed, normal_scale(), &[]),
        Err(SliceError::InvalidInput(
            "classic perimeter raw path coordinate is outside the supported Clipper range".into()
        ))
    );
}

#[test]
fn task22o7_scaled_epsilon_uses_prepared_normal_and_large_bed_scale() {
    let large = CoordinateScale::from_printable_area(&crate::Point2dList(vec![
        crate::Point2d::new(0.0, 0.0),
        crate::Point2d::new(2148.0, 256.0),
    ]));
    assert_eq!(scaled_epsilon(normal_scale()), 100);
    assert_eq!(scaled_epsilon(large), 10);
}
