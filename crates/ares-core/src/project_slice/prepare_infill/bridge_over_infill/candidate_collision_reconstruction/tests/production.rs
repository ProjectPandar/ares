use super::*;

#[test]
fn task22o62_real_no_collision_preserves_initial_allocations_and_uses_no_safety_intersection() {
    let original = vec![rectangle(0, 0, 100, 100)];
    let boundaries = vec![polyline(&[(-10, 0), (110, 0)])];
    let bridge = vec![rectangle(0, 0, 100, 100)];
    let boundary_ptr = boundaries.as_ptr();
    let bridge_ptr = bridge.as_ptr();
    let completed = vec![surface(0, vec![rectangle(108, 0, 120, 100)], 2.0)];
    let mut exact_flow = flow();
    exact_flow.spacing = 0.000_001;

    let output = reconstruct_candidate_bridge_collision(
        &original,
        initial(boundaries, bridge),
        exact_flow,
        0.25,
        &completed,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(output.boundary_polylines.as_ptr(), boundary_ptr);
    assert_eq!(output.bridging_area.as_ptr(), bridge_ptr);
    assert_eq!(output.bridging_angle, 0.25);
}

#[test]
fn task22o62_real_miter_expansion_reaches_the_square_corner_collision() {
    let mut tiny_flow = flow();
    tiny_flow.width = 0.000_01;
    tiny_flow.spacing = 0.000_01;
    let triangle = polygon(&[(0, 0), (100, 0), (50, 120)]);
    let output = reconstruct_candidate_bridge_collision(
        std::slice::from_ref(&triangle),
        initial(
            vec![
                polyline(&[(-100, -10), (200, -10)]),
                polyline(&[(-100, 210), (200, 210)]),
            ],
            vec![triangle.clone()],
        ),
        tiny_flow,
        0.0,
        &[surface(
            0,
            vec![rectangle(48, 180, 52, 190)],
            std::f64::consts::PI * 0.5,
        )],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(output.bridging_angle, std::f64::consts::PI * 0.5);
}

#[test]
fn task22o62_real_collision_reruns_o53_with_first_completed_angle() {
    let original = vec![rectangle(0, 0, 2_000_000, 1_600_000)];
    let boundaries = vec![
        polyline(&[(-500_000, -300_000), (2_500_000, -300_000)]),
        polyline(&[(-500_000, 1_900_000), (2_500_000, 1_900_000)]),
    ];
    let boundary_ptr = boundaries.as_ptr();
    let completed = vec![surface(
        0,
        vec![rectangle(0, 0, 100, 100)],
        std::f64::consts::PI * 0.5,
    )];
    let mut exact_flow = flow();
    exact_flow.spacing = (f64::from(0.4_f32) + 0.05) as f32;
    let original_before = snapshot_polygons(&original);
    let completed_before = completed
        .iter()
        .map(|surface| snapshot_polygons(&surface.new_polygons))
        .collect::<Vec<_>>();

    let first = reconstruct_candidate_bridge_collision(
        &original,
        initial(boundaries, original.clone()),
        exact_flow,
        0.0,
        &completed,
        CoordinateScale::Normal,
    )
    .unwrap();
    let second = reconstruct_candidate_bridge_collision(
        &original,
        initial(
            vec![
                polyline(&[(-500_000, -300_000), (2_500_000, -300_000)]),
                polyline(&[(-500_000, 1_900_000), (2_500_000, 1_900_000)]),
            ],
            original.clone(),
        ),
        exact_flow,
        0.0,
        &completed,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(first.boundary_polylines.as_ptr(), boundary_ptr);
    assert_eq!(
        first.bridging_area,
        vec![polygon(&[
            (1_800_010, 2_300_010),
            (-10, 2_300_010),
            (-10, -700_009),
            (1_800_010, -700_009),
        ])]
    );
    assert_eq!(first.bridging_angle, std::f64::consts::PI * 0.5);
    assert_eq!(second, first);
    assert_eq!(snapshot_polygons(&original), original_before);
    assert_eq!(
        completed
            .iter()
            .map(|surface| snapshot_polygons(&surface.new_polygons))
            .collect::<Vec<_>>(),
        completed_before
    );
}

#[test]
fn task22o62_real_empty_initial_geometry_is_successful_and_keeps_angle() {
    let output = reconstruct_candidate_bridge_collision(
        &[rectangle(0, 0, 100, 100)],
        initial(vec![polyline(&[(0, 0), (100, 0)])], Vec::new()),
        flow(),
        0.75,
        &[surface(0, vec![rectangle(0, 0, 10, 10)], 1.25)],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(output.bridging_area.is_empty());
    assert_eq!(output.bridging_angle, 0.75);
}

#[test]
fn task22o62_natural_range_errors_follow_expand_intersection_then_o53() {
    let far_outside = || polygon(&[(i64::MAX, 0), (0, 1), (0, -1)]);
    let mut tiny_flow = flow();
    tiny_flow.width = 0.000_01;
    tiny_flow.spacing = 0.000_01;
    assert_range_error(reconstruct_candidate_bridge_collision(
        &[rectangle(0, 0, 100, 100)],
        initial(vec![], vec![rectangle(0, 0, 100, 100)]),
        tiny_flow,
        0.0,
        &[surface(0, vec![far_outside()], 1.0)],
        CoordinateScale::Normal,
    ));

    let high = 0x3fff_ffff_ffff_ffff_i64;
    let o53_area = vec![polygon(&[
        (0, high - 2_000),
        (200, high - 2_000),
        (200, high - 1_200),
        (0, high - 1_200),
    ])];
    let o53_boundaries = vec![
        polyline(&[(-100, high - 1_800), (300, high - 1_800)]),
        polyline(&[(-100, high - 600), (300, high - 600)]),
    ];
    let mut o53_flow = flow();
    o53_flow.width = 0.001;
    o53_flow.spacing = 0.000_1;
    assert_range_error(reconstruct_candidate_bridge_collision(
        &o53_area,
        initial(o53_boundaries, vec![rectangle(0, 0, 100, 100)]),
        o53_flow,
        0.0,
        &[surface(
            0,
            vec![rectangle(-300, -300, -250, -250)],
            std::f64::consts::PI * 0.5,
        )],
        CoordinateScale::Normal,
    ));
}
