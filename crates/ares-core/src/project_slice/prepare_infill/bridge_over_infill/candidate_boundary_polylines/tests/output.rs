use super::*;

#[test]
fn task22o59_actual_source_ordered_boundaries_match_exact_literals() {
    let area = candidate_area(
        vec![rectangle(0, 0, 1, 1)],
        vec![rectangle(50_000_000, 0, 60_000_000, 10_000_000)],
    );
    let total = [
        rectangle(0, 0, 20_000_000, 10_000_000),
        rectangle(30_000_000, 0, 40_000_000, 10_000_000),
    ];

    let output = prepare_candidate_boundary_polylines(&area, &total, 10_000_000, 10.0)
        .unwrap()
        .unwrap();
    assert_eq!(
        snapshot_polylines(&output),
        vec![
            vec![
                (-13_000_000, -13_000_000),
                (53_000_000, -13_000_000),
                (53_000_000, 23_000_000),
                (-13_000_000, 23_000_000),
                (-13_000_000, -13_000_000),
            ],
            vec![
                (60_000_003, 10_000_003),
                (49_999_997, 10_000_003),
                (49_999_997, -3),
                (60_000_003, -3),
                (60_000_003, 10_000_003),
            ],
        ]
    );
}

#[test]
fn task22o59_default_miter_three_matches_acute_source_oracle() {
    let area = candidate_area(vec![rectangle(0, 0, 1, 1)], Vec::new());
    let total = [Polygon::new(vec![
        Point::new(0, 0),
        Point::new(100, 0),
        Point::new(0, 100),
    ])];

    let output = prepare_candidate_boundary_polylines(&area, &total, 9, 0.000_01)
        .unwrap()
        .unwrap();
    assert_eq!(
        snapshot_polylines(&output),
        vec![vec![(-12, 128), (-12, -12), (128, -12), (-12, 128)]]
    );
}

#[test]
fn task22o59_repeat_calls_preserve_complete_borrowed_inputs() {
    let area = candidate_area(vec![rectangle(0, 0, 1, 1)], vec![rectangle(30, 0, 40, 10)]);
    let total = vec![rectangle(0, 0, 10, 10)];
    let area_before = (
        area.area_to_be_bridge.as_ptr() as usize,
        area.limiting_area.as_ptr() as usize,
        snapshot_polygons(&area.area_to_be_bridge),
        snapshot_polygons(&area.limiting_area),
    );
    let total_before = (
        total.as_ptr() as usize,
        total[0].points().as_ptr() as usize,
        snapshot_polygons(&total),
    );

    let first = prepare_candidate_boundary_polylines(&area, &total, 10, 10.0).unwrap();
    let second = prepare_candidate_boundary_polylines(&area, &total, 10, 10.0).unwrap();
    assert_eq!(
        first.as_deref().map(snapshot_polylines),
        second.as_deref().map(snapshot_polylines)
    );
    assert_eq!(
        (
            area.area_to_be_bridge.as_ptr() as usize,
            area.limiting_area.as_ptr() as usize,
            snapshot_polygons(&area.area_to_be_bridge),
            snapshot_polygons(&area.limiting_area),
        ),
        area_before
    );
    assert_eq!(
        (
            total.as_ptr() as usize,
            total[0].points().as_ptr() as usize,
            snapshot_polygons(&total),
        ),
        total_before
    );
}
