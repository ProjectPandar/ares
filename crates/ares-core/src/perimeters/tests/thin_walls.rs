use super::*;

#[test]
fn detect_thin_wall_adds_open_external_centerline_for_narrow_rectangle() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 3.0, 0.7)],
    )];
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_detect_thin_wall(true);

    let perimeters = generate_perimeters(&layers, options).unwrap();
    let thin_wall = perimeters[0]
        .paths()
        .iter()
        .find(|path| path.points() == [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)])
        .unwrap();

    assert_eq!(thin_wall.role(), PerimeterRole::External);
    assert!(!thin_wall.is_closed());
}

#[test]
fn detect_thin_wall_disabled_keeps_only_closed_rectangular_perimeter() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 3.0, 0.7)],
    )];
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    );

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert!(perimeters[0].paths().iter().all(PerimeterPath::is_closed));
    assert!(
        !perimeters[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)])
    );
}
