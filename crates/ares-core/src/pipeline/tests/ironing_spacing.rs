use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn ironing_spacing_generates_open_lines_inside_inset_rectangle() {
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
fn smaller_ironing_spacing_increases_rectangle_line_count() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 0.5
    })));

    let y_values = ironing_paths(&finalized[0])
        .into_iter()
        .map(|path| path.points()[0].y())
        .collect::<Vec<_>>();
    assert_eq!(y_values, [0.5, 1.0, 1.5, 2.0, 2.5]);
}

#[test]
fn filament_ironing_spacing_overrides_ordinary_spacing_for_rectangle_lines() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "solid_infill_direction": 0,
        "ironing_inset": 0.5,
        "ironing_spacing": 1.0,
        "filament_ironing_spacing": [0.5]
    })));

    assert_eq!(ironing_paths(&finalized[0]).len(), 5);
}

#[test]
fn nil_filament_ironing_spacing_falls_back_to_ordinary_spacing() {
    for nullable in [json!(["nil", 0.5]), json!([null])] {
        let finalized = finalized_rectangle_paths(options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.5,
            "ironing_spacing": 1.0,
            "filament_ironing_spacing": nullable
        })));

        assert_eq!(ironing_paths(&finalized[0]).len(), 3);
    }
}

#[test]
fn zero_ironing_spacing_keeps_single_inset_rectangle_duplicate() {
    let finalized = finalized_rectangle_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 0.5,
        "ironing_spacing": 0
    })));

    let ironing = ironing_paths(&finalized[0]);
    assert_eq!(ironing.len(), 1);
    assert_eq!(
        ironing[0].points(),
        &[
            Point2::new(0.5, 0.5),
            Point2::new(3.5, 0.5),
            Point2::new(3.5, 2.5),
            Point2::new(0.5, 2.5),
        ]
    );
    assert!(ironing[0].is_closed());
}

#[test]
fn invalid_ironing_spacing_values_reach_slice_error() {
    for (key, value) in [
        ("ironing_spacing", json!(-0.1)),
        ("ironing_spacing", json!(1.1)),
        ("ironing_spacing", json!("NaN")),
        ("ironing_spacing", json!("1.1")),
        ("ironing_spacing", json!("0.5mm")),
        ("ironing_spacing", json!([])),
        ("ironing_spacing", json!({"value": 0.5})),
        ("ironing_spacing", json!(true)),
        ("ironing_spacing", Value::Null),
        ("filament_ironing_spacing", json!(-0.1)),
        ("filament_ironing_spacing", json!(1.1)),
        ("filament_ironing_spacing", json!("NaN")),
        ("filament_ironing_spacing", json!("1.1")),
        ("filament_ironing_spacing", json!("0.5mm")),
        ("filament_ironing_spacing", json!([])),
        ("filament_ironing_spacing", json!({"value": 0.5})),
        ("filament_ironing_spacing", json!(true)),
    ] {
        let mut extra = json!({
            "ironing_type": "top",
            "ironing_inset": 0.5
        });
        extra.as_object_mut().unwrap().insert(key.to_owned(), value);
        let err =
            crate::finalize_print_paths(vec![rectangle_layer()], &options(extra)).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn ordinary_ironing_spacing_does_not_change_support_ironing_duplicate_points() {
    let options = options(json!({
        "support_ironing": true,
        "ironing_spacing": 0.5,
        "filament_ironing_spacing": [0.25]
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
