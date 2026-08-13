use super::*;

#[test]
fn task22o65_real_shrink_and_difference_preserve_ring_topology_and_input() {
    let surfaces = vec![candidate(
        3,
        vec![polygon(&[
            (0, 0),
            (100, 0),
            (100, 100),
            (70, 100),
            (50, 60),
            (30, 100),
            (0, 100),
        ])],
    )];
    let before = candidate_snapshot(&surfaces);
    let polygon_ptr = surfaces[0].new_polygons[0].points().as_ptr();
    let upper = [UpperBridgeEnsuringInput {
        surface: &surfaces[0],
        solid_infill_flow: flow(0.000_01),
    }];

    let first = collect_bridge_rewrite_areas(None, Some(&upper), CoordinateScale::Normal)
        .unwrap()
        .unwrap();
    let second = collect_bridge_rewrite_areas(None, Some(&upper), CoordinateScale::Normal)
        .unwrap()
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(
        snapshot(&first.additional_ensuring_areas),
        vec![
            vec![
                (100, 100),
                (70, 100),
                (50, 60),
                (30, 100),
                (0, 100),
                (0, 0),
                (100, 0),
            ],
            vec![
                (9, 9),
                (9, 91),
                (25, 91),
                (50, 40),
                (76, 91),
                (91, 91),
                (91, 9),
            ],
        ]
    );
    assert_eq!(first.additional_ensuring_areas.len(), 2);
    assert!(first.additional_ensuring_areas[0].area() > 0.0);
    assert!(first.additional_ensuring_areas[1].area() < 0.0);
    assert_eq!(candidate_snapshot(&surfaces), before);
    assert_eq!(surfaces[0].new_polygons[0].points().as_ptr(), polygon_ptr);
}

#[test]
fn task22o65_complete_erosion_returns_the_original_area() {
    let surfaces = [candidate(1, vec![rectangle(0, 0, 100, 100)])];
    let upper = [UpperBridgeEnsuringInput {
        surface: &surfaces[0],
        solid_infill_flow: flow(0.000_06),
    }];
    let output = collect_bridge_rewrite_areas(None, Some(&upper), CoordinateScale::Normal)
        .unwrap()
        .unwrap();

    assert_eq!(output.additional_ensuring_areas.len(), 1);
    assert_eq!(output.additional_ensuring_areas[0].area(), 10_000.0);
}

#[test]
fn task22o65_real_empty_upper_candidate_is_an_empty_success() {
    let surface = candidate(4, Vec::new());
    let upper = [UpperBridgeEnsuringInput {
        surface: &surface,
        solid_infill_flow: flow(0.000_01),
    }];

    let output = collect_bridge_rewrite_areas(None, Some(&upper), CoordinateScale::Normal)
        .unwrap()
        .unwrap();

    assert!(output.cut_from_infill.is_empty());
    assert!(output.additional_ensuring_areas.is_empty());
    assert!(surface.new_polygons.is_empty());
}

#[test]
fn task22o65_natural_offset_range_error_is_atomic() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    let surfaces = vec![candidate(
        5,
        vec![polygon(&[(high, 0), (high - 10, 10), (high - 20, 0)])],
    )];
    let before = candidate_snapshot(&surfaces);
    let upper = [UpperBridgeEnsuringInput {
        surface: &surfaces[0],
        solid_infill_flow: flow(0.000_01),
    }];

    let result = collect_bridge_rewrite_areas(None, Some(&upper), CoordinateScale::Normal);

    assert_eq!(result.unwrap_err(), ClipperError::CoordinateOutOfRange);
    assert_eq!(candidate_snapshot(&surfaces), before);
}
