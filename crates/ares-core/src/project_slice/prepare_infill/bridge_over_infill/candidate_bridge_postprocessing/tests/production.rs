use super::*;

#[test]
fn task22o63_real_morphology_and_booleans_keep_literal_ordered_output() {
    let triangle = polygon(&[(0, 0), (100, 0), (50, 120)]);
    let boundaries = vec![polyline(&[(-100, -10), (200, -10)])];
    let boundary_ptr = boundaries.as_ptr();
    let limiting = vec![rectangle(-200, -200, 200, 250)];
    let fill = limiting.clone();
    let expansion = vec![rectangle(-300, -300, 300, 300)];

    let output = postprocess_candidate_bridge(
        collision(boundaries, vec![triangle], 1.25),
        expansion,
        &limiting,
        &fill,
        &[],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(output.boundary_polylines.as_ptr(), boundary_ptr);
    assert_eq!(
        snapshot(&output.bridging_area),
        vec![vec![(50, 122), (0, 0), (100, 0)]]
    );
    assert_eq!(output.bridging_angle, 1.25);
    assert!(!output.expansion_area.is_empty());
}

#[test]
fn task22o63_real_default_intersection_has_no_safety_offset() {
    let mut exact_flow = flow();
    exact_flow.spacing = 0.000_001;
    let expansion = vec![rectangle(-100, -100, 200, 200)];
    let output = postprocess_candidate_bridge(
        collision(Vec::new(), vec![rectangle(0, 0, 100, 100)], 0.5),
        expansion.clone(),
        &[rectangle(108, 0, 120, 100)],
        &[rectangle(-100, -100, 200, 200)],
        &[],
        exact_flow,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(output.bridging_area.is_empty());
    assert_eq!(
        snapshot(&output.expansion_area),
        vec![vec![(200, 200), (-100, 200), (-100, -100), (200, -100)]]
    );
}

#[test]
fn task22o63_real_range_error_is_atomic_and_preserves_borrowed_inputs() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    let limiting = vec![polygon(&[(high + 1, 0), (0, 1), (0, -1)])];
    let fill = vec![rectangle(-100, -100, 200, 200)];
    let top = vec![rectangle(10, 10, 20, 20)];
    let limiting_before = snapshot(&limiting);
    let fill_before = snapshot(&fill);
    let top_before = snapshot(&top);

    let result = postprocess_candidate_bridge(
        collision(Vec::new(), vec![rectangle(0, 0, 100, 100)], 0.5),
        vec![rectangle(-100, -100, 200, 200)],
        &limiting,
        &fill,
        &top,
        flow(),
        CoordinateScale::Normal,
    );

    assert_eq!(result.unwrap_err(), ClipperError::CoordinateOutOfRange);
    assert_eq!(snapshot(&limiting), limiting_before);
    assert_eq!(snapshot(&fill), fill_before);
    assert_eq!(snapshot(&top), top_before);
}
