use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn omitted_ironing_angle_uses_default_solid_infill_direction() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 3);
    assert_rounded_line(
        ironing[0],
        Point2::new(2.085786, 0.5),
        Point2::new(3.5, 1.914214),
    );
    assert_rounded_line(
        ironing[1],
        Point2::new(0.671573, 0.5),
        Point2::new(2.671573, 2.5),
    );
    assert_rounded_line(
        ironing[2],
        Point2::new(0.5, 1.742641),
        Point2::new(1.257359, 2.5),
    );
}

#[test]
fn ironing_angle_90_generates_vertical_lines() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_angle": 90,
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    assert_open_line(ironing[0], Point2::new(3.5, 0.5), Point2::new(3.5, 2.5));
    assert_open_line(ironing[1], Point2::new(2.5, 0.5), Point2::new(2.5, 2.5));
    assert_open_line(ironing[2], Point2::new(1.5, 0.5), Point2::new(1.5, 2.5));
    assert_open_line(ironing[3], Point2::new(0.5, 0.5), Point2::new(0.5, 2.5));
}

#[test]
fn ironing_angle_numeric_string_reaches_geometry() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_angle": "90",
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    assert_open_line(ironing[0], Point2::new(3.5, 0.5), Point2::new(3.5, 2.5));
    assert_open_line(ironing[3], Point2::new(0.5, 0.5), Point2::new(0.5, 2.5));
}

#[test]
fn legacy_ironing_direction_reaches_ironing_angle_geometry() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_direction": 90,
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    assert_open_line(ironing[0], Point2::new(3.5, 0.5), Point2::new(3.5, 2.5));
    assert_open_line(ironing[3], Point2::new(0.5, 0.5), Point2::new(0.5, 2.5));
}

#[test]
fn legacy_negative_ironing_angle_normalizes_to_zero() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_angle": "-45",
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 3);
    assert_open_line(ironing[0], Point2::new(0.5, 0.5), Point2::new(3.5, 0.5));
    assert_open_line(ironing[2], Point2::new(0.5, 2.5), Point2::new(3.5, 2.5));
}

#[test]
fn solid_infill_direction_90_generates_vertical_lines() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_angle": 0,
        "solid_infill_direction": 90,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    assert_open_line(ironing[0], Point2::new(3.5, 0.5), Point2::new(3.5, 2.5));
    assert_open_line(ironing[3], Point2::new(0.5, 0.5), Point2::new(0.5, 2.5));
}

#[test]
fn non_fixed_ironing_angle_alternates_odd_layer_by_90_degrees() {
    let finalized = finalized_two_solid_layers(options(json!({
        "ironing_type": "solid",
        "ironing_angle": 0,
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let layer_one_ironing = ironing_paths(&finalized[1]);
    assert_eq!(layer_one_ironing.len(), 4);
    assert_open_line(
        layer_one_ironing[0],
        Point2::new(3.5, 0.5),
        Point2::new(3.5, 2.5),
    );
    assert_open_line(
        layer_one_ironing[3],
        Point2::new(0.5, 0.5),
        Point2::new(0.5, 2.5),
    );
}

#[test]
fn solid_infill_rotate_template_controls_layers_without_odd_alternation() {
    let finalized = finalized_two_solid_layers(options(json!({
        "ironing_type": "solid",
        "ironing_angle": 0,
        "solid_infill_direction": 45,
        "solid_infill_rotate_template": "90,0",
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let layer_zero_ironing = ironing_paths(&finalized[0]);
    assert_eq!(layer_zero_ironing.len(), 4);
    assert_open_line(
        layer_zero_ironing[0],
        Point2::new(3.5, 0.5),
        Point2::new(3.5, 2.5),
    );
    let layer_one_ironing = ironing_paths(&finalized[1]);
    assert_eq!(layer_one_ironing.len(), 3);
    assert_open_line(
        layer_one_ironing[0],
        Point2::new(0.5, 0.5),
        Point2::new(3.5, 0.5),
    );
}

#[test]
fn fixed_ironing_angle_suppresses_odd_layer_alternation() {
    let finalized = finalized_two_solid_layers(options(json!({
        "ironing_type": "solid",
        "ironing_angle": 0,
        "ironing_angle_fixed": true,
        "solid_infill_direction": 90,
        "solid_infill_rotate_template": "90",
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let layer_one_ironing = ironing_paths(&finalized[1]);
    assert_eq!(layer_one_ironing.len(), 3);
    assert_open_line(
        layer_one_ironing[0],
        Point2::new(0.5, 0.5),
        Point2::new(3.5, 0.5),
    );
    assert_open_line(
        layer_one_ironing[2],
        Point2::new(0.5, 2.5),
        Point2::new(3.5, 2.5),
    );
}

#[test]
fn invalid_ironing_angle_values_reach_slice_error() {
    for value in [
        json!(360),
        json!(-1),
        json!("NaN"),
        json!("inf"),
        json!("Infinity"),
        json!("0deg"),
        json!(true),
        json!([]),
        json!({ "value": 90 }),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![rectangle_layer(0, PrintPathRole::TopSolidInfill)],
            &options(json!({
                "ironing_type": "top",
                "ironing_angle": value,
                "ironing_inset": 0.5,
                "ironing_spacing": 1.0
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("ironing_angle"));
    }
}

#[test]
fn invalid_ironing_angle_fixed_values_reach_slice_error() {
    for value in [
        json!("true"),
        json!(1),
        json!([]),
        json!({ "value": true }),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![rectangle_layer(0, PrintPathRole::TopSolidInfill)],
            &options(json!({
                "ironing_type": "top",
                "ironing_angle_fixed": value,
                "ironing_inset": 0.5,
                "ironing_spacing": 1.0
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("ironing_angle_fixed"));
    }
}

#[test]
fn concentric_ironing_pattern_ignores_ironing_angle() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_pattern": "concentric",
        "ironing_angle": 90,
        "ironing_inset": 0.5,
        "ironing_spacing": 0.5
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 2);
    assert_closed_loop(
        ironing[0],
        &[
            Point2::new(0.5, 0.5),
            Point2::new(3.5, 0.5),
            Point2::new(3.5, 2.5),
            Point2::new(0.5, 2.5),
        ],
    );
    assert_closed_loop(
        ironing[1],
        &[
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
        ],
    );
}

#[test]
fn ordinary_ironing_angle_does_not_change_support_ironing_duplicate_points() {
    let options = options(json!({
        "support_ironing": true,
        "ironing_angle": 90,
        "ironing_angle_fixed": true,
        "ironing_spacing": 1.0
    }));
    let pipeline = single_path_pipeline(&options, PrintPathRole::SupportMaterialInterface, 1);

    let support_layer = pipeline
        .layer_print_paths()
        .iter()
        .find(|layer| layer.layer_id() == 1)
        .expect("support layer exists");
    let support = support_layer
        .paths()
        .iter()
        .find(|path| path.role() == PrintPathRole::SupportMaterialInterface)
        .expect("support interface path exists");
    let ironing = support_layer
        .paths()
        .iter()
        .find(|path| path.role() == PrintPathRole::Ironing)
        .expect("support ironing path exists");

    assert_eq!(ironing.points(), support.points());
}

fn finalized_rectangle_paths(options: SliceOptions) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(
        vec![rectangle_layer(0, PrintPathRole::TopSolidInfill)],
        &options,
    )
    .unwrap()
}

fn finalized_two_solid_layers(options: SliceOptions) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(
        vec![
            rectangle_layer(0, PrintPathRole::SolidInfill),
            rectangle_layer(1, PrintPathRole::SolidInfill),
        ],
        &options,
    )
    .unwrap()
}

fn rectangle_layer(layer_id: usize, role: PrintPathRole) -> LayerPrintPaths {
    LayerPrintPaths::new(
        layer_id,
        (layer_id + 1) as f64 * 0.2,
        vec![
            PrintPath::new(
                role,
                vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(4.0, 0.0),
                    Point2::new(4.0, 3.0),
                    Point2::new(0.0, 3.0),
                ],
            )
            .unwrap()
            .with_closed(true),
        ],
    )
}

fn ironing_paths(layer: &LayerPrintPaths) -> Vec<&PrintPath> {
    layer
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::Ironing)
        .collect()
}

fn assert_open_line(path: &PrintPath, start: Point2, end: Point2) {
    assert_eq!(path.points(), &[start, end]);
    assert!(!path.is_closed());
}

fn assert_rounded_line(path: &PrintPath, start: Point2, end: Point2) {
    assert_eq!(
        path.points()
            .iter()
            .map(|point| Point2::new(round_6(point.x()), round_6(point.y())))
            .collect::<Vec<_>>(),
        vec![start, end]
    );
    assert!(!path.is_closed());
}

fn assert_closed_loop(path: &PrintPath, points: &[Point2]) {
    assert_eq!(path.points(), points);
    assert!(path.is_closed());
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
