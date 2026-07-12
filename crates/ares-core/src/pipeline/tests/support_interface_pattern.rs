use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

const COORD_EPSILON: f64 = 1e-9;

#[test]
fn omitted_auto_and_rectilinear_patterns_keep_single_interface_family() {
    let omitted = finalize(vec![support_rectangle()], json!({}));
    let auto = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "auto" }),
    );
    let rectilinear = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "rectilinear" }),
    );

    let expected = [
        [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
        [Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
        [Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
    ];
    assert_interface_lines(omitted[0].paths(), &expected);
    assert_interface_lines(auto[0].paths(), &expected);
    assert_interface_lines(rectilinear[0].paths(), &expected);
}

#[test]
fn grid_emits_interface_angle_family_then_base_angle_family() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "grid" }),
    );

    assert_interface_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
            [Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.9), Point2::new(3.0, 1.9)],
        ],
    );
}

#[test]
fn grid_composes_with_support_angle_and_keeps_order() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_angle": 90,
            "support_interface_pattern": "grid"
        }),
    );

    assert_interface_lines(
        finalized[0].paths(),
        &[
            [Point2::new(3.0, 1.0), Point2::new(1.0, 1.0)],
            [Point2::new(3.0, 1.9), Point2::new(1.0, 1.9)],
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.9, 1.0), Point2::new(1.9, 2.0)],
            [Point2::new(2.8, 1.0), Point2::new(2.8, 2.0)],
        ],
    );
}

#[test]
fn grid_uses_support_interface_spacing_pitch_for_both_families() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_pattern": "grid",
            "support_interface_spacing": 0.0
        }),
    );

    assert_eq!(finalized[0].paths().len(), 9);
    assert_points(
        finalized[0].paths()[5].points(),
        &[Point2::new(3.0, 1.0), Point2::new(3.0, 2.0)],
    );
    assert_points(
        finalized[0].paths()[6].points(),
        &[Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
    );
    assert_points(
        finalized[0].paths()[8].points(),
        &[Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
    );
}

#[test]
fn grid_lines_preserve_source_metadata_and_extrusion_role() {
    let source = support_rectangle()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07);
    let finalized = finalize(vec![source], json!({ "support_interface_pattern": "grid" }));

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    for path in finalized[0].paths() {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_eq!(
            path.extrusion_role(),
            Some(PrintPathRole::SupportMaterialInterface)
        );
        assert_eq!(path.effective_layer_height_mm(), Some(0.13));
        assert_eq!(path.unsupported_span_mm(), Some(2.5));
        assert_eq!(path.seam_gap_mm(), 0.07);
        assert!(!path.is_closed());
    }
}

#[test]
fn zero_top_interface_layers_prevents_interface_pattern_conversion() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_pattern": "grid",
            "support_interface_top_layers": 0
        }),
    );

    assert_eq!(finalized[0].paths().len(), 1);
    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterial
    );
    assert_eq!(
        finalized[0].paths()[0].points(),
        [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)]
    );
    assert!(!finalized[0].paths()[0].is_closed());
}

#[test]
fn support_ironing_keeps_solid_interface_but_still_validates_pattern() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_pattern": "concentric",
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
    assert_eq!(support_ironing_count(finalized[0].paths()), 3);

    let err = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![support_rectangle()])],
        &options(json!({
            "support_interface_pattern": "zigzag",
            "support_ironing": true
        })),
    )
    .unwrap_err();
    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_interface_pattern"));
}

#[test]
fn non_target_paths_are_unchanged() {
    let material = PrintPath::new(PrintPathRole::SupportMaterial, rectangle_points()).unwrap();
    let triangle = PrintPath::new(
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
        PrintPath::new(PrintPathRole::SupportMaterialInterface, rectangle_points()).unwrap();
    let solid_rectangle = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);

    let finalized = finalize(
        vec![
            material.clone(),
            triangle.clone(),
            open_rectangle.clone(),
            solid_rectangle.clone(),
        ],
        json!({ "support_interface_pattern": "grid" }),
    );

    assert_eq!(
        finalized[0].paths(),
        [material, triangle, open_rectangle, solid_rectangle]
    );
}

#[test]
fn invalid_pattern_values_reach_slice_error() {
    for value in [
        json!("zigzag"),
        json!(1),
        json!(true),
        Value::Null,
        json!([]),
        json!({ "value": "grid" }),
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(7, 1.6, vec![support_rectangle()])],
            &options(json!({ "support_interface_pattern": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_interface_pattern"));
    }
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
}

fn support_rectangle() -> PrintPath {
    PrintPath::new(PrintPathRole::SupportMaterialInterface, rectangle_points())
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

fn assert_interface_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_points(path.points(), points);
        assert!(!path.is_closed());
    }
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

fn support_ironing_count(paths: &[PrintPath]) -> usize {
    paths
        .iter()
        .filter(|path| path.role() == PrintPathRole::Ironing)
        .count()
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
