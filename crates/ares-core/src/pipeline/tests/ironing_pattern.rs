use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn omitted_ironing_pattern_keeps_rectilinear_lines() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 3);
    assert_open_line(ironing[0], Point2::new(0.5, 0.5), Point2::new(3.5, 0.5));
    assert_open_line(ironing[1], Point2::new(0.5, 1.5), Point2::new(3.5, 1.5));
    assert_open_line(ironing[2], Point2::new(0.5, 2.5), Point2::new(3.5, 2.5));
}

#[test]
fn explicit_rectilinear_ironing_pattern_keeps_rectilinear_lines() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_pattern": "rectilinear",
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 3);
    assert_open_line(ironing[0], Point2::new(0.5, 0.5), Point2::new(3.5, 0.5));
    assert_open_line(ironing[1], Point2::new(0.5, 1.5), Point2::new(3.5, 1.5));
    assert_open_line(ironing[2], Point2::new(0.5, 2.5), Point2::new(3.5, 2.5));
}

#[test]
fn concentric_ironing_pattern_generates_closed_rectangle_loops() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_pattern": "concentric",
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
fn zero_spacing_concentric_pattern_keeps_single_inset_rectangle_duplicate() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_pattern": "concentric",
        "ironing_inset": 0.5,
        "ironing_spacing": 0
    })));

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

#[test]
fn concentric_ironing_pattern_preserves_source_metadata() {
    let source = PrintPath::new(
        PrintPathRole::TopSolidInfill,
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
            "ironing_type": "top",
            "ironing_pattern": "concentric",
            "ironing_inset": 0.5,
            "ironing_spacing": 0.5
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 2);
    for path in ironing {
        assert_eq!(path.role(), PrintPathRole::Ironing);
        assert_eq!(path.effective_layer_height_mm(), Some(0.2));
        assert_eq!(path.unsupported_span_mm(), Some(1.5));
        assert_eq!(path.seam_gap_mm(), 0.04);
        assert!(path.is_closed());
    }
}

#[test]
fn invalid_ironing_pattern_values_reach_slice_error() {
    for value in [
        json!("zig-zag"),
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
                "ironing_type": "top",
                "ironing_pattern": value,
                "ironing_inset": 0.5,
                "ironing_spacing": 0.5
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("ironing_pattern"));
    }
}

#[test]
fn ordinary_ironing_pattern_does_not_change_support_ironing_duplicate_points() {
    let options = options(json!({
        "support_ironing": true,
        "ironing_pattern": "concentric",
        "ironing_spacing": 0.5
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
    crate::finalize_print_paths(vec![rectangle_layer()], &options).unwrap()
}

fn rectangle_layer() -> LayerPrintPaths {
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
