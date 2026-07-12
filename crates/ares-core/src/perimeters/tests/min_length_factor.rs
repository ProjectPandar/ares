use super::*;

#[test]
fn min_length_factor_filters_short_open_thin_wall_on_middle_layer() {
    let perimeters = thin_wall_perimeters(3, 6.0);

    assert!(!has_centerline(perimeters[1].paths()));
}

#[test]
fn min_length_factor_keeps_open_thin_wall_when_length_reaches_threshold() {
    let perimeters = thin_wall_perimeters(3, 5.5);

    let thin_wall = perimeters[1]
        .paths()
        .iter()
        .find(|path| path.points() == thin_wall_centerline())
        .unwrap();

    assert_eq!(thin_wall.role(), PerimeterRole::External);
    assert!(!thin_wall.is_closed());
}

#[test]
fn top_and_first_layers_use_width_half_threshold_instead_of_min_length_factor() {
    let perimeters = thin_wall_perimeters(2, 25.0);

    assert!(has_centerline(perimeters[0].paths()));
    assert!(has_centerline(perimeters[1].paths()));
}

fn thin_wall_perimeters(layer_count: usize, min_length_factor: f64) -> Vec<LayerPerimeters> {
    let layers = (0..layer_count)
        .map(|id| {
            LayerContours::new(
                id,
                0.2 * (id + 1) as f64,
                vec![rectangle(0.0, 0.0, 3.0, 0.7)],
            )
        })
        .collect::<Vec<_>>();
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_detect_thin_wall(true)
    .with_min_length_factor(min_length_factor);

    generate_perimeters(&layers, options).unwrap()
}

fn has_centerline(paths: &[PerimeterPath]) -> bool {
    paths.iter().any(|path| {
        path.role() == PerimeterRole::External && path.points() == thin_wall_centerline()
    })
}

fn thin_wall_centerline() -> [Point2; 2] {
    [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
}
