use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn omitted_ironing_inset_uses_half_first_nozzle_diameter_for_line_path() {
    let finalized = finalized_paths(options(json!({
        "ironing_type": "top",
        "nozzle_diameter": [0.6]
    })));

    assert_eq!(finalized[0].paths().len(), 2);
    let ironing = &finalized[0].paths()[1];
    assert_eq!(ironing.role(), PrintPathRole::Ironing);
    assert_eq!(
        ironing.points(),
        &[Point2::new(0.3, 0.0), Point2::new(1.7, 0.0)]
    );
}

#[test]
fn configured_ironing_inset_shortens_line_path_by_configured_mm() {
    let finalized = finalized_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 0.1,
        "nozzle_diameter": [0.6]
    })));

    assert_eq!(
        finalized[0].paths()[1].points(),
        &[Point2::new(0.1, 0.0), Point2::new(1.9, 0.0)]
    );
}

#[test]
fn configured_ironing_inset_insets_closed_rectangle_loop() {
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
    .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![source])],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.25,
            "ironing_spacing": 0
        })),
    )
    .unwrap();

    assert_eq!(
        finalized[0].paths()[1].points(),
        &[
            Point2::new(0.25, 0.25),
            Point2::new(3.75, 0.25),
            Point2::new(3.75, 2.75),
            Point2::new(0.25, 2.75),
        ]
    );
    assert!(finalized[0].paths()[1].is_closed());
}

#[test]
fn configured_ironing_inset_insets_reversed_closed_rectangle_loop() {
    let source = PrintPath::new(
        PrintPathRole::TopSolidInfill,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 3.0),
            Point2::new(4.0, 3.0),
            Point2::new(4.0, 0.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![source])],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.25,
            "ironing_spacing": 0
        })),
    )
    .unwrap();

    assert_eq!(
        finalized[0].paths()[1].points(),
        &[
            Point2::new(0.25, 0.25),
            Point2::new(0.25, 2.75),
            Point2::new(3.75, 2.75),
            Point2::new(3.75, 0.25),
        ]
    );
}

#[test]
fn unordered_closed_four_corner_path_is_duplicated_unchanged() {
    let points = vec![
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 3.0),
        Point2::new(4.0, 0.0),
        Point2::new(0.0, 3.0),
    ];
    let source = PrintPath::new(PrintPathRole::TopSolidInfill, points.clone())
        .unwrap()
        .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![source])],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.25
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].paths()[1].points(), points);
}

#[test]
fn repeated_first_last_point_polygon_is_duplicated_unchanged() {
    let points = vec![
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 0.0),
        Point2::new(4.0, 3.0),
        Point2::new(0.0, 3.0),
        Point2::new(0.0, 0.0),
    ];
    let source = PrintPath::new(PrintPathRole::TopSolidInfill, points.clone())
        .unwrap()
        .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![source])],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.25
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].paths()[1].points(), points);
}

#[test]
fn repeated_corner_four_point_path_is_duplicated_unchanged() {
    let points = vec![
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 0.0),
        Point2::new(4.0, 0.0),
        Point2::new(0.0, 3.0),
    ];
    let source = PrintPath::new(PrintPathRole::TopSolidInfill, points.clone())
        .unwrap()
        .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![source])],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.25
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].paths()[1].points(), points);
}

#[test]
fn zero_width_four_point_path_is_duplicated_unchanged() {
    let points = vec![
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(1.0, 2.0),
        Point2::new(1.0, 3.0),
    ];
    let source = PrintPath::new(PrintPathRole::TopSolidInfill, points.clone())
        .unwrap()
        .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![source])],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.25
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].paths()[1].points(), points);
}

#[test]
fn too_large_ironing_inset_omits_collapsed_line_duplicate() {
    let finalized = finalized_paths(options(json!({
        "ironing_type": "top",
        "ironing_inset": 1.0
    })));

    assert_eq!(finalized[0].paths().len(), 1);
    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::TopSolidInfill
    );
}

#[test]
fn too_large_ironing_inset_omits_collapsed_rectangle_duplicate() {
    let source = PrintPath::new(
        PrintPathRole::TopSolidInfill,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![source])],
        &options(json!({
            "ironing_type": "top",
            "ironing_inset": 0.5
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].paths().len(), 1);
}

#[test]
fn invalid_ironing_inset_values_reach_slice_error() {
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
                "ironing_inset": value
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("ironing_inset"));
    }
}

#[test]
fn ordinary_ironing_inset_does_not_change_support_ironing_duplicate_points() {
    let options = options(json!({
        "support_ironing": true,
        "ironing_inset": 0.4
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
