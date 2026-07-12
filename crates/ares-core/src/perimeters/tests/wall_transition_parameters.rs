use super::*;

#[test]
fn default_transition_filter_keeps_existing_narrow_arachne_thin_wall() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        wall_generator: WallGenerator::Arachne,
        wall_transition_filter_deviation_percent: 25.0,
        wall_distribution_count: 1,
        height: 0.7,
    });

    assert!(has_centerline(perimeters[0].paths(), narrow_centerline()));
}

#[test]
fn transition_filter_deviation_suppresses_overwide_arachne_proxy() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        wall_generator: WallGenerator::Arachne,
        wall_transition_filter_deviation_percent: 25.0,
        wall_distribution_count: 1,
        height: 1.0,
    });

    assert!(!has_centerline(
        perimeters[0].paths(),
        overwide_centerline()
    ));
}

#[test]
fn larger_transition_filter_deviation_keeps_same_overwide_proxy() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        wall_generator: WallGenerator::Arachne,
        wall_transition_filter_deviation_percent: 100.0,
        wall_distribution_count: 1,
        height: 1.0,
    });
    let thin_wall = centerline_path(perimeters[0].paths(), overwide_centerline());

    assert_eq!(round_6(thin_wall.effective_line_width_mm().unwrap()), 1.0);
}

#[test]
fn wall_distribution_count_spreads_proxy_deviation() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        wall_generator: WallGenerator::Arachne,
        wall_transition_filter_deviation_percent: 25.0,
        wall_distribution_count: 4,
        height: 1.0,
    });

    assert!(has_centerline(perimeters[0].paths(), overwide_centerline()));
}

#[test]
fn classic_preserves_centerline_without_arachne_transition_filter() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        wall_generator: WallGenerator::Classic,
        wall_transition_filter_deviation_percent: 25.0,
        wall_distribution_count: 1,
        height: 1.0,
    });
    let thin_wall = centerline_path(perimeters[0].paths(), overwide_centerline());

    assert_eq!(thin_wall.effective_line_width_mm(), None);
}

struct ThinWallCase {
    wall_generator: WallGenerator,
    wall_transition_filter_deviation_percent: f64,
    wall_distribution_count: u32,
    height: f64,
}

fn thin_wall_perimeters(case: ThinWallCase) -> Vec<LayerPerimeters> {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 3.0, case.height)],
    )];
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_wall_generator(case.wall_generator)
    .with_detect_thin_wall(true)
    .with_wall_transition_filter_deviation_percent(case.wall_transition_filter_deviation_percent)
    .with_wall_distribution_count(case.wall_distribution_count);

    generate_perimeters(&layers, options).unwrap()
}

fn centerline_path(paths: &[PerimeterPath], centerline: [Point2; 2]) -> &PerimeterPath {
    paths
        .iter()
        .find(|path| path.points() == centerline)
        .unwrap()
}

fn has_centerline(paths: &[PerimeterPath], centerline: [Point2; 2]) -> bool {
    paths
        .iter()
        .any(|path| path.role() == PerimeterRole::External && path.points() == centerline)
}

fn narrow_centerline() -> [Point2; 2] {
    [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
}

fn overwide_centerline() -> [Point2; 2] {
    [Point2::new(0.8, 0.5), Point2::new(2.2, 0.5)]
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
