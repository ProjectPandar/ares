use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

#[test]
fn support_ironing_spacing_generates_open_lines_inside_rectangle() {
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
fn smaller_support_ironing_spacing_increases_rectangle_line_count() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_spacing": 0.5
    })));

    let y_values = ironing_paths(&finalized[0])
        .into_iter()
        .map(|path| path.points()[0].y())
        .collect::<Vec<_>>();
    assert_eq!(y_values, [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]);
}

#[test]
fn zero_support_ironing_spacing_keeps_single_closed_duplicate() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_spacing": 0.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 1);
    assert_eq!(ironing[0].points(), rectangle_layer().paths()[0].points());
    assert!(ironing[0].is_closed());
}

#[test]
fn support_ironing_spacing_preserves_support_metadata_on_generated_lines() {
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
            "support_ironing_flow": 25,
            "support_ironing_spacing": 1.0
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 4);
    for path in ironing {
        assert_eq!(path.role(), PrintPathRole::Ironing);
        assert_eq!(path.effective_layer_height_mm(), Some(0.05));
        assert_eq!(path.unsupported_span_mm(), Some(1.5));
        assert_eq!(path.seam_gap_mm(), 0.04);
        assert!(!path.is_closed());
    }
}

#[test]
fn invalid_support_ironing_spacing_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!(1.1),
        json!("NaN"),
        json!("1.1"),
        json!("0.5mm"),
        json!([]),
        json!({"value": 0.5}),
        json!(true),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![rectangle_layer()],
            &options(json!({
                "support_ironing": true,
                "support_ironing_spacing": value
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_ironing_spacing"));
    }
}

#[test]
fn ordinary_ironing_spacing_does_not_control_support_ironing_spacing() {
    let finalized = finalized_rectangle_paths(options(json!({
        "support_ironing": true,
        "support_ironing_spacing": 1.0,
        "ironing_spacing": 0.5,
        "filament_ironing_spacing": [0.25]
    })));

    assert_eq!(ironing_paths(&finalized[0]).len(), 4);
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
