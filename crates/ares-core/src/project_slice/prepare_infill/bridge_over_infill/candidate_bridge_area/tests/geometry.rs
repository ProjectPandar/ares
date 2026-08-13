use super::*;

#[test]
fn task22o58_expands_intersects_filters_and_unions_in_source_order() {
    let candidates = [
        rectangle(0, 0, 1_000, 1_000),
        rectangle(2_000, 0, 3_000, 1_000),
    ];
    let deep = [rectangle(-50, -50, 3_050, 1_050)];
    let unsupported = [rectangle(-100, -100, 1_200, 1_200)];
    let expansion = [rectangle(1_000, 0, 1_500, 1_000)];
    let output = prepare(&candidates, &deep, &unsupported, &expansion, 100);

    assert_eq!(
        snapshot(&output.area_to_be_bridge),
        vec![vec![(1_100, 1_050), (-50, 1_050), (-50, -50), (1_100, -50)]]
    );
    assert_eq!(
        snapshot(&output.limiting_area),
        vec![vec![
            (1_100, 0),
            (1_500, 0),
            (1_500, 1_000),
            (1_100, 1_000),
            (1_100, 1_050),
            (-50, 1_050),
            (-50, -50),
            (1_100, -50),
        ]]
    );
}

#[test]
fn task22o58_empty_survivors_still_normalize_expansion_area() {
    let expansion = [
        rectangle(0, 0, 1_000, 1_000),
        rectangle(500, 0, 1_500, 1_000),
    ];
    let output = prepare(&[], &[], &[], &expansion, 100);

    assert!(output.area_to_be_bridge.is_empty());
    assert_eq!(
        snapshot(&output.limiting_area),
        vec![vec![(0, 0), (1_500, 0), (1_500, 1_000), (0, 1_000)]]
    );
}

#[test]
fn task22o58_empty_unsupported_area_filters_every_intersection_polygon() {
    let candidates = [rectangle(0, 0, 1_000, 1_000)];
    let deep = [rectangle(-1_000, -1_000, 2_000, 2_000)];
    let expansion = [rectangle(3_000, 0, 4_000, 1_000)];
    let output = prepare(&candidates, &deep, &[], &expansion, 100);

    assert!(output.area_to_be_bridge.is_empty());
    assert_eq!(total_area(&output.limiting_area), 1_000_000.0);
}

#[test]
fn task22o58_deep_intersection_can_split_one_candidate_and_filter_each_piece() {
    let candidates = [rectangle(0, 0, 3_000, 1_000)];
    let deep = [
        rectangle(0, -100, 1_000, 1_100),
        rectangle(2_000, -100, 3_000, 1_100),
    ];
    let unsupported = [rectangle(1_900, -200, 3_100, 1_200)];
    let output = prepare(&candidates, &deep, &unsupported, &[], 100);

    assert_eq!(
        snapshot(&output.area_to_be_bridge),
        vec![vec![
            (3_000, 1_100),
            (2_000, 1_100),
            (2_000, -100),
            (3_000, -100)
        ]]
    );
    assert_eq!(
        snapshot(&output.area_to_be_bridge),
        snapshot(&output.limiting_area)
    );
}
