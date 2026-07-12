use super::*;

#[test]
fn arachne_wall_simplification_removes_short_low_deviation_vertex() {
    let options = options(json!({
        "wall_generator": "arachne",
        "wall_maximum_resolution": 0.5,
        "wall_maximum_deviation": 0.05
    }));

    assert_eq!(external_points(options), simplified_points());
}

#[test]
fn wall_simplification_preserves_vertex_when_resolution_is_too_small() {
    let options = options(json!({
        "wall_generator": "arachne",
        "wall_maximum_resolution": 0.2,
        "wall_maximum_deviation": 0.05
    }));

    assert_eq!(external_points(options), notched_points());
}

#[test]
fn wall_simplification_preserves_vertex_when_deviation_is_too_small() {
    let options = options(json!({
        "wall_generator": "arachne",
        "wall_maximum_resolution": 0.5,
        "wall_maximum_deviation": 0.005
    }));

    assert_eq!(external_points(options), notched_points());
}

#[test]
fn wall_simplification_repeats_after_last_vertex_removal() {
    let options = options(json!({
        "wall_generator": "arachne",
        "wall_maximum_resolution": 0.5,
        "wall_maximum_deviation": 0.05
    }));
    let stable_points = vec![
        Point2::new(0.0, 0.0),
        Point2::new(-0.022547, 0.294552),
        Point2::new(1.0, 1.0),
        Point2::new(-0.064466, -0.388626),
        Point2::new(-0.085721, -0.324485),
    ];

    let points = external_points_for(stable_points, options);

    assert_eq!(points.len(), 3);
    assert!(points.contains(&Point2::new(-0.022547, 0.294552)));
    assert!(points.contains(&Point2::new(1.0, 1.0)));
    assert!(points.contains(&Point2::new(-0.064466, -0.388626)));
}

#[test]
fn classic_wall_generator_preserves_current_compatibility_geometry() {
    let options = options(json!({
        "wall_generator": "classic",
        "wall_maximum_resolution": 0.5,
        "wall_maximum_deviation": 0.05
    }));

    assert_eq!(external_points(options), notched_points());
}

#[test]
fn simplification_runs_before_fuzzy_skin_points_are_generated() {
    let options = options(json!({
        "wall_generator": "arachne",
        "wall_maximum_resolution": 0.5,
        "wall_maximum_deviation": 0.05,
        "fuzzy_skin": "external",
        "fuzzy_skin_first_layer": true,
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 0.1
    }));

    let points = external_points(options);

    assert_ne!(points, simplified_points());
    assert!(points.len() > simplified_points().len());
}

fn external_points(options: SliceOptions) -> Vec<Point2> {
    external_points_for(notched_points(), options)
}

fn external_points_for(points: Vec<Point2>, options: SliceOptions) -> Vec<Point2> {
    let layers = [LayerContours::new(0, 0.2, vec![Contour::new(points)])];
    generate_perimeters(&layers, options.perimeter_options().unwrap()).unwrap()[0].paths()[0]
        .points()
        .to_vec()
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "wall_loops": 1,
        "line_width": 0.4,
        "wall_sequence": "outer wall/inner wall",
        "seam_position": "aligned"
    });
    for (key, extra_value) in extra.as_object().unwrap() {
        value[key] = extra_value.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn notched_points() -> Vec<Point2> {
    vec![
        Point2::new(0.0, 0.0),
        Point2::new(0.3, 0.02),
        Point2::new(0.6, 0.0),
        Point2::new(0.6, 1.0),
        Point2::new(0.0, 1.0),
    ]
}

fn simplified_points() -> Vec<Point2> {
    vec![
        Point2::new(0.0, 0.0),
        Point2::new(0.6, 0.0),
        Point2::new(0.6, 1.0),
        Point2::new(0.0, 1.0),
    ]
}
