use super::*;
use crate::SliceOptions;
use serde_json::json;

#[test]
fn external_fuzzy_skin_replaces_external_points_after_first_layer() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
    ];
    let options = options(json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 0.3,
        "fuzzy_skin_first_layer": false
    }))
    .perimeter_options()
    .unwrap();

    let perimeters = generate_perimeters(&layers, options).unwrap();
    let first = external_points(&perimeters[0]);
    let second = external_points(&perimeters[1]);

    assert_eq!(first, rectangle_points(0.0, 0.0, 4.0, 4.0));
    assert_ne!(second, rectangle_points(0.0, 0.0, 4.0, 4.0));
    assert!(second.len() > 4);
    assert!(second.iter().all(|point| point.x() >= -0.2
        && point.x() <= 4.2
        && point.y() >= -0.2
        && point.y() <= 4.2));
}

#[test]
fn fuzzy_skin_first_layer_option_allows_layer_zero_fuzzification() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let options = options(json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_first_layer": true
    }))
    .perimeter_options()
    .unwrap();

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_ne!(
        external_points(&perimeters[0]),
        rectangle_points(0.0, 0.0, 4.0, 4.0)
    );
}

#[test]
fn disabled_hole_and_effective_disabled_fuzzy_skin_preserve_geometry() {
    for extra in [
        json!({ "fuzzy_skin": "disabled_fuzzy" }),
        json!({ "fuzzy_skin": "none" }),
        json!({ "fuzzy_skin": "hole", "fuzzy_skin_first_layer": true }),
        json!({ "fuzzy_skin": "external", "fuzzy_skin_first_layer": true, "fuzzy_skin_point_distance": 0.0 }),
        json!({ "fuzzy_skin": "external", "fuzzy_skin_first_layer": true, "fuzzy_skin_thickness": 0.0 }),
    ] {
        let layers = [LayerContours::new(
            0,
            0.2,
            vec![rectangle(0.0, 0.0, 4.0, 4.0)],
        )];
        let options = options(extra).perimeter_options().unwrap();

        let perimeters = generate_perimeters(&layers, options).unwrap();

        assert_eq!(
            external_points(&perimeters[0]),
            rectangle_points(0.0, 0.0, 4.0, 4.0)
        );
    }
}

#[test]
fn all_fuzzifies_external_but_leaves_internal_walls_unchanged() {
    let layer = fuzzy_wall_layer(json!({ "fuzzy_skin": "all" }));

    assert_ne!(
        external_points(&layer),
        rectangle_points(0.0, 0.0, 4.0, 4.0)
    );
    assert_eq!(internal_points(&layer), rectangular_internal_points());
}

#[test]
fn external_fuzzy_skin_leaves_internal_walls_unchanged() {
    let layer = fuzzy_wall_layer(json!({ "fuzzy_skin": "external" }));

    assert_ne!(
        external_points(&layer),
        rectangle_points(0.0, 0.0, 4.0, 4.0)
    );
    assert_eq!(internal_points(&layer), rectangular_internal_points());
}

#[test]
fn allwalls_fuzzifies_generated_internal_walls() {
    let layer = fuzzy_wall_layer(json!({
        "fuzzy_skin": "allwalls",
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 0.3
    }));

    assert_fuzzy_internal(internal_points(&layer));
}

#[test]
fn allwalls_internal_walls_respect_first_layer_gate() {
    let layer = fuzzy_wall_layer(json!({
        "fuzzy_skin": "allwalls",
        "fuzzy_skin_first_layer": false
    }));

    assert_eq!(internal_points(&layer), rectangular_internal_points());
}

#[test]
fn allwalls_ripple_noise_fuzzifies_internal_walls() {
    let layer = fuzzy_wall_layer(json!({
        "fuzzy_skin": "allwalls",
        "fuzzy_skin_noise_type": "ripple",
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 1.0,
        "fuzzy_skin_ripples_per_layer": 1,
        "fuzzy_skin_ripple_offset": 0,
        "fuzzy_skin_layers_between_ripple_offset": 1
    }));

    assert_fuzzy_internal(internal_points(&layer));
}

#[test]
fn ripple_noise_type_generates_orca_arc_length_wave_points() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let options = options(json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_first_layer": true,
        "fuzzy_skin_noise_type": "ripple",
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 1.0,
        "fuzzy_skin_ripples_per_layer": 1,
        "fuzzy_skin_ripple_offset": 0,
        "fuzzy_skin_layers_between_ripple_offset": 1
    }))
    .perimeter_options()
    .unwrap();

    let perimeters = generate_perimeters(&layers, options).unwrap();
    let points = external_points(&perimeters[0]);

    assert_eq!(points.len(), 16);
    assert_point_close(points[0], Point2::new(0.0, 0.0));
    assert_point_close(points[1], Point2::new(1.0, 0.076536686));
    assert_point_close(points[2], Point2::new(2.0, 0.141421356));
    assert_point_close(points[3], Point2::new(3.0, 0.184775907));
    assert_point_close(points[4], Point2::new(3.8, 0.0));
}

#[test]
fn ripple_noise_type_is_not_classic_random_for_same_fixture() {
    let layers = [LayerContours::new(
        1,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let base = json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 1.0,
        "fuzzy_skin_ripples_per_layer": 1,
        "fuzzy_skin_ripple_offset": 0,
        "fuzzy_skin_layers_between_ripple_offset": 1
    });

    let mut classic = base.clone();
    classic["fuzzy_skin_noise_type"] = json!("classic");
    let classic_points = external_points(
        &generate_perimeters(&layers, options(classic).perimeter_options().unwrap()).unwrap()[0],
    );

    let mut ripple = base;
    ripple["fuzzy_skin_noise_type"] = json!("ripple");
    let ripple_points = external_points(
        &generate_perimeters(&layers, options(ripple).perimeter_options().unwrap()).unwrap()[0],
    );

    assert_ne!(classic_points, ripple_points);
    assert_point_close(ripple_points[1], Point2::new(1.0, 0.076536686));
}

#[test]
fn ripple_parameters_change_wave_frequency_and_layer_phase() {
    let one_ripple = ripple_points(json!({
        "fuzzy_skin_ripples_per_layer": 1,
        "fuzzy_skin_ripple_offset": 0,
        "fuzzy_skin_layers_between_ripple_offset": 1
    }));
    let two_ripples = ripple_points(json!({
        "fuzzy_skin_ripples_per_layer": 2,
        "fuzzy_skin_ripple_offset": 0,
        "fuzzy_skin_layers_between_ripple_offset": 1
    }));
    assert_eq!(one_ripple.len(), two_ripples.len());
    assert_point_close(one_ripple[2], Point2::new(2.0, 0.141421356));
    assert_point_close(two_ripples[2], Point2::new(2.0, 0.2));

    let shifted_every_layer = ripple_layer_points(
        1,
        json!({
            "fuzzy_skin_ripples_per_layer": 1,
            "fuzzy_skin_ripple_offset": "50%",
            "fuzzy_skin_layers_between_ripple_offset": 1
        }),
    );
    let held_first_group = ripple_layer_points(
        1,
        json!({
            "fuzzy_skin_ripples_per_layer": 1,
            "fuzzy_skin_ripple_offset": "50%",
            "fuzzy_skin_layers_between_ripple_offset": 2
        }),
    );
    assert_point_close(shifted_every_layer[1], Point2::new(1.0, -0.076536686));
    assert_point_close(held_first_group[1], Point2::new(1.0, 0.076536686));
}

#[test]
fn fuzzy_skin_rejects_malformed_enum_and_out_of_range_numbers() {
    for extra in [
        json!({ "fuzzy_skin": "painted" }),
        json!({ "fuzzy_skin_noise_type": "unknown" }),
        json!({ "fuzzy_skin_thickness": -0.1 }),
        json!({ "fuzzy_skin_thickness": 2.1 }),
        json!({ "fuzzy_skin_thickness": "NaN" }),
        json!({ "fuzzy_skin_thickness": "inf" }),
        json!({ "fuzzy_skin_point_distance": -0.1 }),
        json!({ "fuzzy_skin_point_distance": 5.1 }),
        json!({ "fuzzy_skin_point_distance": "NaN" }),
        json!({ "fuzzy_skin_point_distance": "inf" }),
        json!({ "fuzzy_skin_ripples_per_layer": 0 }),
        json!({ "fuzzy_skin_ripples_per_layer": "many" }),
        json!({ "fuzzy_skin_ripple_offset": -0.1 }),
        json!({ "fuzzy_skin_ripple_offset": "101%" }),
        json!({ "fuzzy_skin_ripple_offset": "wide" }),
        json!({ "fuzzy_skin_layers_between_ripple_offset": 0 }),
        json!({ "fuzzy_skin_layers_between_ripple_offset": "many" }),
    ] {
        assert!(matches!(
            options(extra).perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

fn external_points(layer: &LayerPerimeters) -> Vec<Point2> {
    layer
        .paths()
        .iter()
        .find(|path| path.role() == PerimeterRole::External)
        .unwrap()
        .points()
        .to_vec()
}

fn internal_points(layer: &LayerPerimeters) -> Vec<Point2> {
    layer
        .paths()
        .iter()
        .find(|path| path.role() == PerimeterRole::Internal)
        .unwrap()
        .points()
        .to_vec()
}

fn fuzzy_wall_layer(extra: serde_json::Value) -> LayerPerimeters {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let mut value = json!({
        "wall_loops": 2,
        "fuzzy_skin_first_layer": true
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    generate_perimeters(&layers, options(value).perimeter_options().unwrap())
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

fn rectangular_internal_points() -> Vec<Point2> {
    rectangle_points(0.4, 0.4, 3.6, 3.6)
}

fn assert_fuzzy_internal(internal: Vec<Point2>) {
    assert_ne!(internal, rectangular_internal_points());
    assert!(internal.len() > 4);
    assert!(
        internal.iter().all(|point| point.x() >= 0.2
            && point.x() <= 3.8
            && point.y() >= 0.2
            && point.y() <= 3.8)
    );
}

fn ripple_points(extra: serde_json::Value) -> Vec<Point2> {
    ripple_layer_points(0, extra)
}

fn ripple_layer_points(layer_id: usize, extra: serde_json::Value) -> Vec<Point2> {
    let layers = [LayerContours::new(
        layer_id,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let mut value = json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_first_layer": true,
        "fuzzy_skin_noise_type": "ripple",
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 1.0
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    let options = options(value).perimeter_options().unwrap();
    let perimeters = generate_perimeters(&layers, options).unwrap();
    external_points(&perimeters[0])
}

fn assert_point_close(actual: Point2, expected: Point2) {
    const EPSILON: f64 = 1e-6;
    assert!(
        (actual.x() - expected.x()).abs() <= EPSILON,
        "x {} != {} for {:?}",
        actual.x(),
        expected.x(),
        actual
    );
    assert!(
        (actual.y() - expected.y()).abs() <= EPSILON,
        "y {} != {} for {:?}",
        actual.y(),
        expected.y(),
        actual
    );
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
