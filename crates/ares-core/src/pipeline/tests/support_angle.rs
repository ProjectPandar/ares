use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

const COORD_EPSILON: f64 = 1e-9;

#[test]
fn omitted_support_angle_keeps_base_horizontal_and_interface_vertical() {
    let finalized = finalize(
        vec![
            support_rectangle(PrintPathRole::SupportMaterial),
            support_rectangle(PrintPathRole::SupportMaterialInterface),
        ],
        json!({}),
    );

    assert_points(
        finalized[0].paths()[0].points(),
        &[Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
    );
    assert_points(
        finalized[0].paths()[1].points(),
        &[Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
    );
    assert_points(
        finalized[0].paths()[2].points(),
        &[Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
    );
}

#[test]
fn explicit_ninety_rotates_base_vertical_and_interface_horizontal() {
    let finalized = finalize(
        vec![
            support_rectangle(PrintPathRole::SupportMaterial),
            support_rectangle(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "support_angle": 90,
            "support_base_pattern_spacing": 0.0,
            "support_interface_spacing": 0.0
        }),
    );

    assert_points(
        finalized[0].paths()[0].points(),
        &[Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
    );
    assert_points(
        finalized[0].paths()[1].points(),
        &[Point2::new(1.4, 1.0), Point2::new(1.4, 2.0)],
    );
    assert_points(
        finalized[0].paths()[2].points(),
        &[Point2::new(1.8, 1.0), Point2::new(1.8, 2.0)],
    );
    assert_points(
        finalized[0].paths()[6].points(),
        &[Point2::new(3.0, 1.0), Point2::new(1.0, 1.0)],
    );
}

#[test]
fn numeric_string_angle_generates_expected_diagonal_chords() {
    let finalized = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_angle": "45",
            "support_base_pattern_spacing": 0.6
        }),
    );

    assert_eq!(finalized[0].paths().len(), 2);
    assert_points(
        finalized[0].paths()[0].points(),
        &[
            Point2::new(1.585786437626905, 1.0),
            Point2::new(2.585786437626905, 2.0),
        ],
    );
    assert_points(
        finalized[0].paths()[1].points(),
        &[
            Point2::new(1.0, 1.8284271247461903),
            Point2::new(1.1715728752538097, 2.0),
        ],
    );
}

#[test]
fn generated_lines_preserve_source_metadata() {
    let source = support_rectangle(PrintPathRole::SupportMaterial)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07);
    let finalized = finalize(vec![source], json!({ "support_angle": 90 }));

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    for path in finalized[0].paths() {
        assert_eq!(path.role(), PrintPathRole::SupportMaterial);
        assert_eq!(path.extrusion_role(), Some(PrintPathRole::SupportMaterial));
        assert_eq!(path.effective_layer_height_mm(), Some(0.13));
        assert_eq!(path.unsupported_span_mm(), Some(2.5));
        assert_eq!(path.seam_gap_mm(), 0.07);
        assert!(!path.is_closed());
    }
}

#[test]
fn zero_top_interface_layers_uses_base_angle_after_role_conversion() {
    let finalized = finalize(
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "support_angle": 90,
            "support_interface_top_layers": 0,
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterial
    );
    assert_points(
        finalized[0].paths()[0].points(),
        &[Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
    );
    assert_eq!(finalized[0].paths()[0].extrusion_role(), None);
}

#[test]
fn support_ironing_preserves_solid_interface_before_ironing_lines() {
    let finalized = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterialInterface)],
        json!({
            "support_angle": 45,
            "support_ironing": true,
            "support_ironing_spacing": 0.5
        }),
    );

    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterialInterface
    );
    assert!(finalized[0].paths()[0].is_closed());
    assert_eq!(finalized[0].paths()[0].points(), rectangle_points());
}

#[test]
fn non_rectangular_non_closed_and_non_support_paths_are_unchanged() {
    let triangle = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(2.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let interface_triangle = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(2.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let open_rectangle =
        PrintPath::new(PrintPathRole::SupportMaterial, rectangle_points()).unwrap();
    let solid_rectangle = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);
    let finalized = finalize(
        vec![
            triangle.clone(),
            interface_triangle.clone(),
            open_rectangle.clone(),
            solid_rectangle.clone(),
        ],
        json!({ "support_angle": 45 }),
    );

    assert_eq!(
        finalized[0].paths(),
        [
            triangle,
            interface_triangle,
            open_rectangle,
            solid_rectangle
        ]
    );
}

#[test]
fn invalid_support_angle_values_reach_slice_error() {
    for value in [
        json!(360),
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("45deg"),
        json!(true),
        Value::Null,
        json!([]),
        json!({ "value": 45 }),
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(
                1,
                0.4,
                vec![support_rectangle(PrintPathRole::SupportMaterial)],
            )],
            &options(json!({ "support_angle": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_angle"));
    }
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, rectangle_points())
        .unwrap()
        .with_closed(true)
}

fn rectangle_points() -> Vec<Point2> {
    vec![
        Point2::new(1.0, 1.0),
        Point2::new(3.0, 1.0),
        Point2::new(3.0, 2.0),
        Point2::new(1.0, 2.0),
    ]
}

fn assert_points(actual: &[Point2], expected: &[Point2]) {
    assert_eq!(actual.len(), expected.len());
    for (actual_point, expected_point) in actual.iter().zip(expected) {
        assert!(
            (actual_point.x() - expected_point.x()).abs() <= COORD_EPSILON,
            "x mismatch: actual {actual_point:?}, expected {expected_point:?}"
        );
        assert!(
            (actual_point.y() - expected_point.y()).abs() <= COORD_EPSILON,
            "y mismatch: actual {actual_point:?}, expected {expected_point:?}"
        );
    }
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
