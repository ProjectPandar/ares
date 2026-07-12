use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn filament_ironing_inset_overrides_ordinary_ironing_inset_for_line_path() {
    let finalized = finalized_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 0.4,
        "filament_ironing_inset": [0.1],
        "nozzle_diameter": [0.6]
    })));

    assert_eq!(finalized[0].paths().len(), 2);
    let ironing = &finalized[0].paths()[1];
    assert_eq!(ironing.role(), PrintPathRole::Ironing);
    assert_eq!(
        ironing.points(),
        &[Point2::new(0.1, 0.0), Point2::new(1.9, 0.0)]
    );
}

#[test]
fn scalar_string_filament_ironing_inset_overrides_ordinary_inset() {
    let finalized = finalized_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 0.4,
        "filament_ironing_inset": "0.2",
        "nozzle_diameter": [0.6]
    })));

    assert_eq!(
        finalized[0].paths()[1].points(),
        &[Point2::new(0.2, 0.0), Point2::new(1.8, 0.0)]
    );
}

#[test]
fn nil_filament_ironing_inset_falls_back_to_ordinary_ironing_inset() {
    let finalized = finalized_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 0.4,
        "filament_ironing_inset": ["nil", 0.1],
        "nozzle_diameter": [0.6]
    })));

    assert_eq!(
        finalized[0].paths()[1].points(),
        &[Point2::new(0.4, 0.0), Point2::new(1.6, 0.0)]
    );
}

#[test]
fn zero_filament_ironing_inset_uses_half_first_nozzle_diameter() {
    let finalized = finalized_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 0.1,
        "filament_ironing_inset": 0,
        "nozzle_diameter": [0.6]
    })));

    assert_eq!(
        finalized[0].paths()[1].points(),
        &[Point2::new(0.3, 0.0), Point2::new(1.7, 0.0)]
    );
}

#[test]
fn invalid_filament_ironing_inset_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!(100.1),
        json!("NaN"),
        json!("101"),
        json!("0.25mm"),
        json!("wide"),
        json!([]),
        json!({"value": 0.2}),
        json!(true),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(
                0,
                0.2,
                vec![
                    PrintPath::new(
                        PrintPathRole::TopSolidInfill,
                        vec![Point2::new(0.0, 0.0), Point2::new(2.0, 0.0)],
                    )
                    .unwrap(),
                ],
            )],
            &options(json!({
                "ironing_type": "top",
                "filament_ironing_inset": value
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_ironing_inset"));
    }
}

#[test]
fn filament_ironing_inset_does_not_change_support_ironing_duplicate_points() {
    let options = options(json!({
        "support_ironing": true,
        "filament_ironing_inset": 0.4
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

fn finalized_paths(options: SliceOptions) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![
                PrintPath::new(
                    PrintPathRole::TopSolidInfill,
                    vec![Point2::new(0.0, 0.0), Point2::new(2.0, 0.0)],
                )
                .unwrap(),
            ],
        )],
        &options,
    )
    .unwrap()
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
