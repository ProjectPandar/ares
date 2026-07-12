use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

#[test]
fn omitted_support_ironing_pattern_keeps_rectilinear_lines() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    assert_open_line(ironing[0], Point2::new(0.0, 0.0), Point2::new(4.0, 0.0));
    assert_open_line(ironing[1], Point2::new(0.0, 1.0), Point2::new(4.0, 1.0));
    assert_open_line(ironing[2], Point2::new(0.0, 2.0), Point2::new(4.0, 2.0));
    assert_open_line(ironing[3], Point2::new(0.0, 3.0), Point2::new(4.0, 3.0));
}

#[test]
fn explicit_rectilinear_support_ironing_pattern_keeps_rectilinear_lines() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_pattern": "rectilinear",
        "support_ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    assert_open_line(ironing[0], Point2::new(0.0, 0.0), Point2::new(4.0, 0.0));
    assert_open_line(ironing[1], Point2::new(0.0, 1.0), Point2::new(4.0, 1.0));
    assert_open_line(ironing[2], Point2::new(0.0, 2.0), Point2::new(4.0, 2.0));
    assert_open_line(ironing[3], Point2::new(0.0, 3.0), Point2::new(4.0, 3.0));
}

#[test]
fn concentric_support_ironing_pattern_generates_closed_rectangle_loops() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_pattern": "concentric",
        "support_ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 2);
    assert_closed_loop(
        ironing[0],
        &[
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 3.0),
            Point2::new(0.0, 3.0),
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
fn zero_spacing_concentric_support_ironing_keeps_single_closed_duplicate() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_pattern": "concentric",
        "support_ironing_spacing": 0.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 1);
    assert_closed_loop(ironing[0], rectangle_layer().paths()[0].points());
}

#[test]
fn concentric_support_ironing_pattern_preserves_support_metadata() {
    let source = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 3.0),
            Point2::new(0.0, 3.0),
        ],
    )
    .unwrap()
    .with_closed(true)
    .with_effective_layer_height_mm(0.2)
    .with_unsupported_span_mm(Some(1.5))
    .with_seam_gap_mm(0.04);

    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![source])],
        &options(json!({
            "support_ironing": true,
            "support_ironing_pattern": "concentric",
            "support_ironing_flow": 25,
            "support_ironing_spacing": 1.0
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 2);
    for path in ironing {
        assert_eq!(path.role(), PrintPathRole::Ironing);
        assert_eq!(
            path.extrusion_role(),
            Some(PrintPathRole::SupportMaterialInterface)
        );
        assert_eq!(path.effective_layer_height_mm(), Some(0.05));
        assert_eq!(path.unsupported_span_mm(), Some(1.5));
        assert_eq!(path.seam_gap_mm(), 0.04);
        assert!(path.is_closed());
    }
}

#[test]
fn invalid_support_ironing_pattern_values_reach_slice_error() {
    for value in [
        json!("monotonic"),
        json!("concentric "),
        json!(""),
        json!(true),
        json!([]),
        json!({ "value": "concentric" }),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![rectangle_layer()],
            &options(json!({
                "support_ironing": true,
                "support_ironing_pattern": value,
                "support_ironing_spacing": 1.0
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_ironing_pattern"));
    }
}

#[test]
fn legacy_zig_zag_support_ironing_pattern_normalizes_to_rectilinear() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_pattern": "zig-zag",
        "support_ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    assert_open_line(ironing[0], Point2::new(0.0, 0.0), Point2::new(4.0, 0.0));
    assert_open_line(ironing[3], Point2::new(0.0, 3.0), Point2::new(4.0, 3.0));
}

#[test]
fn support_ironing_pattern_does_not_change_ordinary_ironing_duplicate_points() {
    let finalized = crate::finalize_print_paths(
        vec![ordinary_rectangle_layer()],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.5,
            "ironing_spacing": 0.0,
            "support_ironing_pattern": "concentric"
        })),
    )
    .unwrap();

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 1);
    assert_closed_loop(
        ironing[0],
        &[
            Point2::new(0.5, 0.5),
            Point2::new(3.5, 0.5),
            Point2::new(3.5, 2.5),
            Point2::new(0.5, 2.5),
        ],
    );
}

fn finalized_rectangle_paths(options: SliceOptions) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![rectangle_layer()], &options).unwrap()
}

fn rectangle_layer() -> LayerPrintPaths {
    LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::SupportMaterialInterface,
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

fn ordinary_rectangle_layer() -> LayerPrintPaths {
    LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::TopSolidInfill,
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

fn assert_closed_loop(path: &PrintPath, points: &[Point2]) {
    assert_eq!(path.points(), points);
    assert!(path.is_closed());
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
