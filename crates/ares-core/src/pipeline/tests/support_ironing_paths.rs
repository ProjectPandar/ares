use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
    gcode::format_gcode, pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn omitted_support_ironing_preserves_support_interface_without_ironing() {
    let options = options(json!({}));
    let output = output_for_layer(&options, 1);

    assert_eq!(
        extrusion_comment_count(&output, "support_material_interface"),
        1
    );
    assert_eq!(extrusion_comment_count(&output, "ironing"), 0);
}

#[test]
fn disabled_support_ironing_preserves_support_interface_without_ironing() {
    let options = options(json!({ "support_ironing": false }));
    let output = output_for_layer(&options, 1);

    assert_eq!(
        extrusion_comment_count(&output, "support_material_interface"),
        1
    );
    assert_eq!(extrusion_comment_count(&output, "ironing"), 0);
}

#[test]
fn enabled_support_ironing_emits_interface_then_ironing() {
    let options = options(json!({ "support_ironing": true }));
    let output = output_for_layer(&options, 1);

    assert!(
        output.find(";EXTRUSION:print:support_material_interface:")
            < output.find(";EXTRUSION:print:support_ironing:")
    );
    assert_eq!(
        extrusion_comment_count(&output, "support_material_interface"),
        1
    );
    assert_eq!(extrusion_comment_count(&output, "support_ironing"), 1);
    assert_eq!(extrusion_comment_count(&output, "ironing"), 0);
    assert!(output.contains(";PRINT_PATH:support_ironing:"));
    assert!(output.contains(";MOVE:print:support_ironing:"));
}

#[test]
fn support_ironing_path_uses_ironing_speed_feedrate() {
    let options = options(json!({
        "support_ironing": true,
        "support_interface_speed": 37,
        "ironing_speed": 15,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    assert!(output.contains(";SPEED:print:support_ironing:1,0:900"));
    assert!(output.contains(" F900"));
}

#[test]
fn ordinary_ironing_flow_does_not_control_support_ironing_delta() {
    let low = options(json!({
        "support_ironing": true,
        "support_ironing_flow": 25,
        "ironing_flow": 10,
        "filament_ironing_flow": [10],
        "slow_down_for_layer_cooling": false
    }));
    let high = options(json!({
        "support_ironing": true,
        "support_ironing_flow": 25,
        "ironing_flow": 80,
        "filament_ironing_flow": [80],
        "slow_down_for_layer_cooling": false
    }));

    assert_delta_eq(
        first_support_ironing_delta(&output_for_layer(&high, 1)),
        first_support_ironing_delta(&output_for_layer(&low, 1)),
    );
}

#[test]
fn support_ironing_flow_controls_support_ironing_delta() {
    let options = options(json!({
        "support_ironing": true,
        "support_ironing_flow": 25,
        "slow_down_for_layer_cooling": false
    }));

    let expected = options
        .extrusion_options()
        .unwrap()
        .extrusion_delta_for_segment(PrintPathRole::SupportMaterialInterface, 0.05, false, 1.0)
        .unwrap();

    assert_delta_eq(
        first_support_ironing_delta(&output_for_layer(&options, 1)),
        expected,
    );
}

#[test]
fn support_ironing_flow_zero_keeps_path_with_zero_delta() {
    let output = output_for_layer(
        &options(json!({
            "support_ironing": true,
            "support_ironing_flow": 0,
            "slow_down_for_layer_cooling": false
        })),
        1,
    );

    assert_eq!(extrusion_comment_count(&output, "support_ironing"), 1);
    assert_delta_eq(first_support_ironing_delta(&output), 0.0);
}

#[test]
fn support_ironing_flow_does_not_change_independent_ironing_paths() {
    let low = first_ironing_delta(&output_for_role(
        &options(json!({
            "ironing_flow": 25,
            "support_ironing_flow": 80,
            "slow_down_for_layer_cooling": false
        })),
        PrintPathRole::Ironing,
        1,
    ));
    let high = first_ironing_delta(&output_for_role(
        &options(json!({
            "ironing_flow": 50,
            "support_ironing_flow": 80,
            "slow_down_for_layer_cooling": false
        })),
        PrintPathRole::Ironing,
        1,
    ));

    assert_delta_eq(high, low * 2.0);
}

#[test]
fn invalid_support_ironing_flow_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!(100.1),
        json!("NaN"),
        json!("101"),
        json!("25%"),
        json!("fast"),
        json!([]),
        json!({"value": 25}),
        json!(true),
        Value::Null,
    ] {
        let options = options(json!({
            "support_ironing": true,
            "support_ironing_flow": value
        }));
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(
                1,
                0.4,
                vec![
                    PrintPath::new(
                        PrintPathRole::SupportMaterialInterface,
                        vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
                    )
                    .unwrap(),
                ],
            )],
            &options,
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_ironing_flow"));
    }
}

#[test]
fn support_ironing_duplicate_scales_effective_height_by_default_flow() {
    let source = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![Point2::new(1.0, 2.0), Point2::new(3.0, 4.0)],
    )
    .unwrap()
    .with_effective_layer_height_mm(0.13);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![source])],
        &options(json!({ "support_ironing": true })),
    )
    .unwrap();

    assert_eq!(
        finalized[0].paths()[1].effective_layer_height_mm(),
        Some(0.013)
    );
}

#[test]
fn support_ironing_duplicate_uses_fallback_layer_height_when_source_has_none() {
    let finalized = crate::finalize_print_paths(
        vec![
            LayerPrintPaths::new(0, 0.2, Vec::new()),
            LayerPrintPaths::new(
                1,
                0.4,
                vec![
                    PrintPath::new(
                        PrintPathRole::SupportMaterialInterface,
                        vec![Point2::new(1.0, 2.0), Point2::new(3.0, 4.0)],
                    )
                    .unwrap(),
                ],
            ),
        ],
        &options(json!({
            "support_ironing": true,
            "support_ironing_flow": 25
        })),
    )
    .unwrap();

    assert_eq!(
        finalized[1].paths()[1].effective_layer_height_mm(),
        Some(0.05),
    );
}

#[test]
fn support_ironing_duplicate_preserves_source_path_metadata() {
    let source = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![Point2::new(1.0, 2.0), Point2::new(3.0, 4.0)],
    )
    .unwrap()
    .with_effective_layer_height_mm(0.13)
    .with_unsupported_span_mm(Some(2.5))
    .with_seam_gap_mm(0.07)
    .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![source.clone()])],
        &options(json!({ "support_ironing": true })),
    )
    .unwrap();

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    assert_eq!(finalized[0].paths().len(), 2);
    assert_eq!(finalized[0].paths()[0], source);
    let ironing = &finalized[0].paths()[1];
    assert_eq!(ironing.role(), PrintPathRole::Ironing);
    assert_eq!(ironing.points(), source.points());
    assert_eq!(ironing.effective_layer_height_mm(), Some(0.013));
    assert_eq!(ironing.unsupported_span_mm(), source.unsupported_span_mm());
    assert_eq!(ironing.seam_gap_mm(), source.seam_gap_mm());
    assert_eq!(ironing.is_closed(), source.is_closed());
}

#[test]
fn invalid_support_ironing_values_reach_slice_error() {
    for value in [json!(1), json!("true"), json!([]), Value::Null] {
        let err = options(json!({ "support_ironing": value }))
            .support_ironing()
            .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_ironing"));
    }
}

fn output_for_layer(options: &SliceOptions, layer_id: usize) -> String {
    output_for_role(options, PrintPathRole::SupportMaterialInterface, layer_id)
}

fn output_for_role(options: &SliceOptions, role: PrintPathRole, layer_id: usize) -> String {
    let pipeline = single_path_pipeline(options, role, layer_id);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
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

fn extrusion_comment_count(output: &str, role: &str) -> usize {
    let prefix = format!(";EXTRUSION:print:{role}:");
    output
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .count()
}

fn first_support_ironing_delta(gcode: &str) -> f64 {
    first_role_delta(gcode, "support_ironing")
}

fn first_ironing_delta(gcode: &str) -> f64 {
    first_role_delta(gcode, "ironing")
}

fn first_role_delta(gcode: &str, target_role: &str) -> f64 {
    let mut previous_e = 0.0;
    let target_prefix = format!(";EXTRUSION:print:{target_role}:");
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(&target_prefix) {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing {target_role} extrusion");
}

fn assert_delta_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 0.000002);
}
