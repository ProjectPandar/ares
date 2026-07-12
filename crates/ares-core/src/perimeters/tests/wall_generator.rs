use super::*;

#[test]
fn default_wall_generator_keeps_current_rectangular_perimeter_geometry() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wall_loops": 3,
        "line_width": 0.4
    }))
    .unwrap();
    let perimeter_options = options.perimeter_options().unwrap();

    assert_eq!(perimeter_options.wall_generator(), WallGenerator::Arachne);
    assert_eq!(
        generated_path_points(perimeter_options),
        current_rectangular_path_points()
    );
}

#[test]
fn classic_and_arachne_currently_share_the_compatibility_geometry() {
    let classic = options("classic").perimeter_options().unwrap();
    let arachne = options("arachne").perimeter_options().unwrap();

    assert_eq!(classic.wall_generator(), WallGenerator::Classic);
    assert_eq!(arachne.wall_generator(), WallGenerator::Arachne);
    assert_eq!(
        generated_path_points(classic),
        generated_path_points(arachne)
    );
}

fn options(wall_generator: &str) -> SliceOptions {
    serde_json::from_value(json!({
        "wall_generator": wall_generator,
        "wall_loops": 3,
        "line_width": 0.4
    }))
    .unwrap()
}

fn generated_path_points(options: PerimeterOptions) -> Vec<Vec<Point2>> {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    generate_perimeters(&layers, options).unwrap()[0]
        .paths()
        .iter()
        .map(|path| path.points().to_vec())
        .collect()
}

fn current_rectangular_path_points() -> Vec<Vec<Point2>> {
    vec![
        rectangle_points(
            0.7570796326794897,
            0.7570796326794897,
            3.2429203673205103,
            3.2429203673205103,
        ),
        rectangle_points(0.4, 0.4, 3.6, 3.6),
        rectangle_points(0.0, 0.0, 4.0, 4.0),
    ]
}
