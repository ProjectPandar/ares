use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
    gcode::format_gcode, pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn omitted_top_interface_layers_preserves_support_interface_gcode() {
    let output = output_for_layer(&options(json!({
        "support_interface_speed": 37,
        "support_speed": 91,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    })));

    assert_eq!(count_role(&output, "support_material_interface"), 1);
    assert_eq!(count_role(&output, "support_material"), 0);
    assert!(output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:5460"));
}

#[test]
fn zero_top_interface_layers_routes_existing_interface_path_to_support_material() {
    let output = output_for_layer(&options(json!({
        "support_interface_top_layers": 0,
        "support_interface_speed": 37,
        "support_speed": 91,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    })));

    assert_eq!(count_role(&output, "support_material_interface"), 0);
    assert_eq!(count_role(&output, "support_material"), 1);
    assert!(output.contains(";SPEED:print:support_material:1,0:5460"));
    assert!(!output.contains(";SPEED:print:support_material_interface:1,0:2220"));
}

#[test]
fn zero_top_positive_bottom_layers_preserves_existing_interface_path() {
    let output = output_for_layer(&options(json!({
        "support_interface_top_layers": 0,
        "support_interface_bottom_layers": 1,
        "support_interface_speed": 37,
        "support_speed": 91,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    })));

    assert_eq!(count_role(&output, "support_material_interface"), 1);
    assert_eq!(count_role(&output, "support_material"), 0);
    assert!(output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:5460"));
}

#[test]
fn zero_top_bottom_same_as_top_routes_existing_interface_path_to_support_material() {
    let output = output_for_layer(&options(json!({
        "support_interface_top_layers": 0,
        "support_interface_bottom_layers": -1,
        "support_interface_speed": 37,
        "support_speed": 91,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    })));

    assert_eq!(count_role(&output, "support_material_interface"), 0);
    assert_eq!(count_role(&output, "support_material"), 1);
    assert!(output.contains(";SPEED:print:support_material:1,0:5460"));
    assert!(!output.contains(";SPEED:print:support_material_interface:1,0:2220"));
}

#[test]
fn positive_top_zero_bottom_layers_preserves_existing_interface_path() {
    let output = output_for_layer(&options(json!({
        "support_interface_top_layers": 2,
        "support_interface_bottom_layers": 0,
        "support_interface_speed": 37,
        "support_speed": 91,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    })));

    assert_eq!(count_role(&output, "support_material_interface"), 1);
    assert_eq!(count_role(&output, "support_material"), 0);
    assert!(output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:5460"));
}

#[test]
fn positive_top_interface_layers_accept_decimal_integer_forms() {
    for value in [json!(2), json!(2.0), json!("2"), json!("2.0")] {
        let output = output_for_layer(&options(json!({
            "support_interface_top_layers": value,
            "support_interface_speed": 37,
            "support_speed": 91,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        })));

        assert_eq!(count_role(&output, "support_material_interface"), 1);
        assert_eq!(count_role(&output, "support_material"), 0);
        assert!(output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    }
}

#[test]
fn bottom_interface_layers_accept_decimal_integer_forms() {
    for value in [json!(1), json!(1.0), json!("1"), json!("1.0"), json!("-1")] {
        let output = output_for_layer(&options(json!({
            "support_interface_top_layers": 1,
            "support_interface_bottom_layers": value,
            "support_interface_speed": 37,
            "support_speed": 91,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        })));

        assert_eq!(count_role(&output, "support_material_interface"), 1);
        assert_eq!(count_role(&output, "support_material"), 0);
        assert!(output.contains(";SPEED:print:support_material_interface:1,0:2220"));
    }
}

#[test]
fn zero_top_interface_layers_uses_support_flow_instead_of_interface_flow() {
    let low = options(json!({
        "support_interface_top_layers": 0,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": 0.5,
        "support_interface_flow_ratio": 1.75,
        "slow_down_for_layer_cooling": false
    }));
    let high = options(json!({
        "support_interface_top_layers": 0,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": 1.5,
        "support_interface_flow_ratio": 0.25,
        "slow_down_for_layer_cooling": false
    }));
    let low_output = output_for_layer(&low);
    let high_output = output_for_layer(&high);

    assert_delta_eq(
        first_extrusion_delta_for_role(&high_output, "support_material"),
        first_extrusion_delta_for_role(&low_output, "support_material") * 3.0,
    );
}

#[test]
fn zero_top_interface_layers_uses_layer_fan_baseline_instead_of_interface_fan() {
    let output = output_for_layer(&options(json!({
        "support_interface_top_layers": 0,
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "support_material_interface_fan_speed": 65
    })));

    assert_eq!(fan_lines(&output), vec!["M106 S102"]);
    assert_line_before(&output, "M106 S102", ";EXTRUSION:print:support_material:");
    assert!(!output.contains("M106 S166"));
    assert!(!output.contains(";EXTRUSION:print:support_material_interface:"));
}

#[test]
fn zero_top_interface_layers_prevents_support_ironing_duplicate() {
    let output = output_for_layer(&options(json!({
        "support_interface_top_layers": 0,
        "support_ironing": true,
        "support_interface_speed": 37,
        "support_speed": 91,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    })));

    assert_eq!(count_role(&output, "support_material_interface"), 0);
    assert_eq!(count_role(&output, "support_material"), 1);
    assert_eq!(count_role(&output, "ironing"), 0);
}

#[test]
fn invalid_top_interface_layer_values_reach_slice_error() {
    for value in [
        json!(-1),
        json!(0.5),
        json!("0.5"),
        json!("NaN"),
        json!("3%"),
        json!("fast"),
        json!("999999999999999999999999999999999999.0"),
        json!([]),
        json!({"value": 0}),
        json!(true),
        Value::Null,
    ] {
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
            &options(json!({ "support_interface_top_layers": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_interface_top_layers"));
    }
}

#[test]
fn invalid_bottom_interface_layer_values_reach_slice_error() {
    for value in [
        json!(-2),
        json!(0.5),
        json!("0.5"),
        json!("NaN"),
        json!("3%"),
        json!("fast"),
        json!("999999999999999999999999999999999999.0"),
        json!([]),
        json!({"value": 0}),
        json!(true),
        Value::Null,
    ] {
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
            &options(json!({ "support_interface_bottom_layers": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_interface_bottom_layers"));
    }
}

#[test]
fn zero_top_and_bottom_interface_layers_preserve_path_geometry_and_metadata() {
    let source = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![Point2::new(1.0, 2.0), Point2::new(3.0, 4.0)],
    )
    .unwrap()
    .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
    .with_effective_layer_height_mm(0.13)
    .with_unsupported_span_mm(Some(2.5))
    .with_seam_gap_mm(0.07)
    .with_closed(true);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![source.clone()])],
        &options(json!({
            "support_interface_top_layers": 0,
            "support_interface_bottom_layers": 0
        })),
    )
    .unwrap();

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    assert_eq!(finalized[0].paths().len(), 1);
    let rewritten = &finalized[0].paths()[0];
    assert_eq!(rewritten.role(), PrintPathRole::SupportMaterial);
    assert_eq!(rewritten.extrusion_role(), None);
    assert_eq!(rewritten.points(), source.points());
    assert_eq!(
        rewritten.effective_layer_height_mm(),
        source.effective_layer_height_mm()
    );
    assert_eq!(
        rewritten.unsupported_span_mm(),
        source.unsupported_span_mm()
    );
    assert_eq!(rewritten.seam_gap_mm(), source.seam_gap_mm());
    assert_eq!(rewritten.is_closed(), source.is_closed());
}

fn output_for_layer(options: &SliceOptions) -> String {
    let pipeline = single_path_pipeline(options, PrintPathRole::SupportMaterialInterface, 1);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn count_role(output: &str, role: &str) -> usize {
    let prefix = format!(";EXTRUSION:print:{role}:");
    output
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .count()
}

fn first_extrusion_delta_for_role(gcode: &str, role: &str) -> f64 {
    let mut previous_e = 0.0;
    let target = format!("{role}:");
    for original in gcode.lines() {
        let Some(rest) = original.strip_prefix(";EXTRUSION:print:") else {
            continue;
        };
        let Some((role_and_segment, e)) = rest.rsplit_once(':') else {
            continue;
        };
        let Ok(e) = e.parse::<f64>() else {
            continue;
        };
        if role_and_segment.starts_with(&target) {
            return e - previous_e;
        }
        previous_e = e;
    }
    panic!("missing extrusion role {role}");
}

fn fan_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M106 "))
        .collect()
}

fn assert_line_before(output: &str, first: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines.iter().position(|line| *line == first).unwrap();
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap();
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn assert_delta_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 0.000002);
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
