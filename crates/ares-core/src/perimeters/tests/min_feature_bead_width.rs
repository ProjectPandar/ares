use super::*;

#[test]
fn arachne_suppresses_open_thin_wall_below_min_feature_size() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        target_layer: 0,
        wall_generator: WallGenerator::Arachne,
        detect_thin_wall: true,
        min_feature_size_percent: 200.0,
        initial_layer_min_bead_width_percent: 85.0,
        min_bead_width_percent: 85.0,
        wall_transition_filter_deviation_percent: 25.0,
        height: 0.7,
    });

    assert!(!has_centerline(perimeters[0].paths()));
}

#[test]
fn classic_preserves_open_thin_wall_below_min_feature_size() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        target_layer: 0,
        wall_generator: WallGenerator::Classic,
        detect_thin_wall: true,
        min_feature_size_percent: 200.0,
        initial_layer_min_bead_width_percent: 85.0,
        min_bead_width_percent: 85.0,
        wall_transition_filter_deviation_percent: 25.0,
        height: 0.7,
    });

    assert!(has_centerline(perimeters[0].paths()));
}

#[test]
fn first_layer_uses_initial_layer_min_bead_width_for_surviving_thin_wall() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        target_layer: 0,
        wall_generator: WallGenerator::Arachne,
        detect_thin_wall: true,
        min_feature_size_percent: 100.0,
        initial_layer_min_bead_width_percent: 200.0,
        min_bead_width_percent: 85.0,
        wall_transition_filter_deviation_percent: 25.0,
        height: 0.7,
    });
    let thin_wall = centerline_path(perimeters[0].paths());

    assert_eq!(thin_wall.effective_line_width_mm(), Some(0.8));
}

#[test]
fn later_layer_uses_min_bead_width_for_surviving_thin_wall() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        target_layer: 1,
        wall_generator: WallGenerator::Arachne,
        detect_thin_wall: true,
        min_feature_size_percent: 100.0,
        initial_layer_min_bead_width_percent: 85.0,
        min_bead_width_percent: 175.0,
        wall_transition_filter_deviation_percent: 25.0,
        height: 0.7,
    });
    let thin_wall = centerline_path(perimeters[1].paths());

    assert_eq!(round_6(thin_wall.effective_line_width_mm().unwrap()), 0.7);
}

#[test]
fn detect_thin_wall_disabled_leaves_closed_rectangular_geometry_unchanged() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        target_layer: 0,
        wall_generator: WallGenerator::Arachne,
        detect_thin_wall: false,
        min_feature_size_percent: 200.0,
        initial_layer_min_bead_width_percent: 200.0,
        min_bead_width_percent: 175.0,
        wall_transition_filter_deviation_percent: 25.0,
        height: 0.7,
    });

    assert!(perimeters[0].paths().iter().all(PerimeterPath::is_closed));
    assert!(!has_centerline(perimeters[0].paths()));
}

#[test]
fn surviving_thin_wall_uses_thickness_when_above_min_bead_width() {
    let perimeters = thin_wall_perimeters(ThinWallCase {
        target_layer: 0,
        wall_generator: WallGenerator::Arachne,
        detect_thin_wall: true,
        min_feature_size_percent: 100.0,
        initial_layer_min_bead_width_percent: 85.0,
        min_bead_width_percent: 85.0,
        wall_transition_filter_deviation_percent: 100.0,
        height: 0.9,
    });
    let thin_wall = perimeters[0]
        .paths()
        .iter()
        .find(|path| path.points() == thick_thin_wall_centerline())
        .unwrap();

    assert_eq!(round_6(thin_wall.effective_line_width_mm().unwrap()), 0.9);
}

struct ThinWallCase {
    target_layer: usize,
    wall_generator: WallGenerator,
    detect_thin_wall: bool,
    min_feature_size_percent: f64,
    initial_layer_min_bead_width_percent: f64,
    min_bead_width_percent: f64,
    wall_transition_filter_deviation_percent: f64,
    height: f64,
}

fn thin_wall_perimeters(case: ThinWallCase) -> Vec<LayerPerimeters> {
    let layers = (0..=case.target_layer)
        .map(|id| {
            LayerContours::new(
                id,
                0.2 * (id + 1) as f64,
                vec![rectangle(0.0, 0.0, 3.0, case.height)],
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
    .with_wall_generator(case.wall_generator)
    .with_detect_thin_wall(case.detect_thin_wall)
    .with_min_feature_size_percent(case.min_feature_size_percent)
    .with_initial_layer_min_bead_width_percent(case.initial_layer_min_bead_width_percent)
    .with_min_bead_width_percent(case.min_bead_width_percent)
    .with_wall_transition_filter_deviation_percent(case.wall_transition_filter_deviation_percent);

    generate_perimeters(&layers, options).unwrap()
}

fn centerline_path(paths: &[PerimeterPath]) -> &PerimeterPath {
    paths
        .iter()
        .find(|path| path.points() == thin_wall_centerline())
        .unwrap()
}

fn has_centerline(paths: &[PerimeterPath]) -> bool {
    paths.iter().any(|path| {
        path.role() == PerimeterRole::External && path.points() == thin_wall_centerline()
    })
}

fn thin_wall_centerline() -> [Point2; 2] {
    [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
}

fn thick_thin_wall_centerline() -> [Point2; 2] {
    [Point2::new(0.8, 0.45), Point2::new(2.2, 0.45)]
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
