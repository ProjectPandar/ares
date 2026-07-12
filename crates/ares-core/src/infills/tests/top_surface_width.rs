use super::*;
use crate::SliceOptions;
use serde_json::json;

#[test]
fn min_width_top_surface_filters_narrow_rectangular_top_surface() {
    let layers = vec![
        narrow_rectangular_layer_with_id(0, 0.2),
        narrow_rectangular_layer_with_id(1, 0.4),
    ];
    let options = top_surface_width_options(json!({ "min_width_top_surface": 1.0 }));

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[1].paths().is_empty());
}

#[test]
fn min_width_top_surface_keeps_rectangular_top_surface_at_threshold() {
    let layers = vec![
        threshold_rectangular_layer_with_id(0, 0.2),
        threshold_rectangular_layer_with_id(1, 0.4),
    ];
    let options = top_surface_width_options(json!({ "min_width_top_surface": 1.0 }));

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(!infills[1].paths().is_empty());
}

#[test]
fn min_width_top_surface_filters_valid_cyclic_start_and_reverse_winding_rectangles() {
    for top_layer in [
        rotated_start_rectangular_layer_with_id(1, 0.4),
        reverse_winding_rectangular_layer_with_id(1, 0.4),
    ] {
        let layers = vec![narrow_rectangular_layer_with_id(0, 0.2), top_layer];
        let options = top_surface_width_options(json!({ "min_width_top_surface": 1.0 }));

        let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

        assert!(infills[1].paths().is_empty());
    }
}

#[test]
fn min_width_top_surface_zero_preserves_current_top_surface_output() {
    let layers = vec![square_layer_with_id(0, 0.2), square_layer_with_id(1, 0.4)];
    let options = top_surface_width_options(json!({ "min_width_top_surface": 0 }));

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[1].paths().len(), 4);
}

#[test]
fn min_width_top_surface_preserves_non_rectangular_top_contours() {
    let layers = vec![
        non_rectangular_layer_with_id(0, 0.2),
        non_rectangular_layer_with_id(1, 0.4),
    ];
    let options = top_surface_width_options(json!({ "min_width_top_surface": 10.0 }));

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(!infills[1].paths().is_empty());
}

#[test]
fn min_width_top_surface_does_not_filter_bottom_or_sparse_or_internal_solid_roles() {
    let layers = vec![
        narrow_rectangular_layer_with_id(0, 0.2),
        narrow_rectangular_layer_with_id(1, 0.4),
        narrow_rectangular_layer_with_id(2, 0.6),
    ];
    let dense_options = top_surface_width_options(json!({
        "sparse_infill_density": 100,
        "bottom_shell_layers": 1,
        "top_shell_layers": 1,
        "min_width_top_surface": 1.0
    }));
    let sparse_options = top_surface_width_options(json!({
        "sparse_infill_density": 50,
        "bottom_shell_layers": 0,
        "top_shell_layers": 1,
        "min_width_top_surface": 1.0
    }));

    let dense_infills = generate_infills(&print_layers(&layers), &layers, dense_options).unwrap();
    let sparse_infills = generate_infills(&print_layers(&layers), &layers, sparse_options).unwrap();

    assert!(!dense_infills[0].paths().is_empty());
    assert!(!dense_infills[1].paths().is_empty());
    assert!(dense_infills[2].paths().is_empty());
    assert!(!sparse_infills[0].paths().is_empty());
    assert!(!sparse_infills[1].paths().is_empty());
    assert!(sparse_infills[2].paths().is_empty());
}

fn top_surface_width_options(extra: serde_json::Value) -> InfillOptions {
    let mut value = json!({
        "nozzle_diameter": [0.4],
        "line_width": 0.5,
        "sparse_infill_density": 50,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "solid_infill_direction": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0,
        "wall_loops": 0
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    let options: SliceOptions = serde_json::from_value(value).unwrap();
    options.infill_options().unwrap()
}

fn square_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(layer_id, print_z, square_layer().contours().to_vec())
}

fn narrow_rectangular_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 0.8),
            Point2::new(0.0, 0.8),
        ])],
    )
}

fn threshold_rectangular_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 1.0),
            Point2::new(0.0, 1.0),
        ])],
    )
}

fn rotated_start_rectangular_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 0.8),
            Point2::new(0.0, 0.8),
            Point2::new(0.0, 0.0),
        ])],
    )
}

fn reverse_winding_rectangular_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 0.8),
            Point2::new(4.0, 0.8),
            Point2::new(4.0, 0.0),
        ])],
    )
}

fn non_rectangular_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 0.8),
            Point2::new(2.0, 1.0),
            Point2::new(0.0, 0.8),
        ])],
    )
}
