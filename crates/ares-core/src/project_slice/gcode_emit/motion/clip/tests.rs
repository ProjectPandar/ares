#[test]
fn clips_a_closed_loop_from_its_end() {
    let mut points = vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 0)];

    super::clip_end(&mut points, 1.0);

    assert_eq!(points, vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 1)]);
}

#[test]
fn negative_fractional_endpoint_truncates_toward_zero() {
    // Upstream `Polyline::clip_end` emplaces
    // `(last_point + v * factor).cast<coord_t>()` (`Polyline.cpp:67`) —
    // Eigen's per-component `static_cast` truncates toward zero: raw
    // y = -6892499.659 truncates to -6892499, not -6892500.
    let mut points = vec![(-7_549_495, -6_469_541), (-7_099_181, -6_920_814)];

    super::clip_end(&mut points, 40_000.0);

    assert_eq!(points[1], (-7_127_435, -6_892_499));
}

#[test]
fn clips_across_short_terminal_segments() {
    let mut points = vec![(0, 0), (2, 0), (2, 2), (0, 2), (0, 0)];

    super::clip_end(&mut points, 3.0);

    assert_eq!(points, vec![(0, 0), (2, 0), (2, 2), (1, 2)]);
}
