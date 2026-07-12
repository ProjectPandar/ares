use super::*;

#[test]
fn only_one_wall_top_limits_topmost_layer_to_external_wall() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(2, 0.6, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
    ];
    let options = PerimeterOptions::new(
        3,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_only_one_wall_top(true);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(perimeters[0].paths().len(), 3);
    assert_eq!(perimeters[1].paths().len(), 3);
    assert_eq!(perimeters[2].paths().len(), 1);
    assert_eq!(perimeters[2].paths()[0].role(), PerimeterRole::External);
}

#[test]
fn only_one_wall_top_defaults_to_full_wall_count() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
    ];
    let options = PerimeterOptions::new(
        3,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    );

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(perimeters[1].paths().len(), 3);
}

#[test]
fn only_one_wall_top_does_not_synthesize_zero_wall_loops() {
    let layers = [LayerContours::new(
        2,
        0.6,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let options = PerimeterOptions::new(
        0,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_only_one_wall_top(true);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert!(perimeters[0].paths().is_empty());
}

#[test]
fn only_one_wall_top_wins_after_alternate_extra_wall_on_odd_top_layer() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
        LayerContours::new(2, 0.6, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
        LayerContours::new(3, 0.8, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
    ];
    let options = PerimeterOptions::new(
        2,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_alternate_extra_wall(true)
    .with_sparse_infill_density_percent(20.0)
    .with_only_one_wall_top(true);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(perimeters[0].paths().len(), 2);
    assert_eq!(perimeters[1].paths().len(), 3);
    assert_eq!(perimeters[2].paths().len(), 2);
    assert_eq!(perimeters[3].paths().len(), 1);
    assert_eq!(perimeters[3].paths()[0].role(), PerimeterRole::External);
}
