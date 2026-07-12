use super::*;
use crate::{
    LayerPrintPaths, Point2, PrintPath, gcode::format_gcode,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn pipeline_rejects_invalid_support_interface_not_for_body_before_model_loading() {
    for value in [json!("true"), json!("false"), json!(1), Value::Null] {
        let options = options(json!({ "support_interface_not_for_body": value }));
        let err = run_slicing_pipeline(b"not a model", &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_interface_not_for_body"));
    }
}

#[test]
fn valid_support_interface_not_for_body_values_preserve_current_pipeline_artifacts() {
    let baseline_options = options(json!({ "support_interface_not_for_body": true }));
    let baseline = run_slicing_pipeline(square_pyramid_ascii_stl(), &baseline_options).unwrap();
    let baseline_gcode =
        String::from_utf8(format_gcode(&baseline, &baseline_options).unwrap()).unwrap();

    let tuned_options = options(json!({ "support_interface_not_for_body": false }));
    let tuned = run_slicing_pipeline(square_pyramid_ascii_stl(), &tuned_options).unwrap();
    let tuned_gcode = String::from_utf8(format_gcode(&tuned, &tuned_options).unwrap()).unwrap();

    assert_eq!(tuned.layer_print_paths(), baseline.layer_print_paths());
    assert_eq!(
        tuned.layer_toolpath_moves(),
        baseline.layer_toolpath_moves()
    );
    assert_eq!(
        tuned.layer_extrusion_moves(),
        baseline.layer_extrusion_moves()
    );
    assert_eq!(tuned.layer_speed_moves(), baseline.layer_speed_moves());
    assert_eq!(tuned.diagnostics(), baseline.diagnostics());
    assert_eq!(tuned_gcode, baseline_gcode);
}

#[test]
fn support_interface_not_for_body_avoids_fixed_first_interface_selector_in_support_body_gcode() {
    let shared = support_body_options(json!(false));
    let avoided = support_body_options(json!(true));
    let shared_output = support_body_output(&shared);
    let avoided_output = support_body_output(&avoided);

    assert!(shared_output.contains(";EXTRUSION:print:support_material:"));
    assert!(avoided_output.contains(";EXTRUSION:print:support_material:"));
    assert_ne!(avoided_output, shared_output);
    assert!(
        first_support_material_delta(&avoided_output)
            < first_support_material_delta(&shared_output)
    );
}

#[test]
fn support_interface_not_for_body_preserves_closed_support_body_print_paths() {
    let shared = support_body_options(json!(false));
    let avoided = support_body_options(json!(true));

    assert_eq!(
        closed_support_body_paths(&avoided),
        closed_support_body_paths(&shared)
    );
}

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn support_body_options(not_for_body: Value) -> SliceOptions {
    options(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "support_filament": 0,
        "support_interface_filament": 1,
        "support_interface_not_for_body": not_for_body,
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0,
        "support_line_width": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
}

fn support_body_output(options: &SliceOptions) -> String {
    let pipeline = single_path_pipeline(options, PrintPathRole::SupportMaterial, 1);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn closed_support_body_paths(options: &SliceOptions) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            1,
            0.4,
            vec![
                PrintPath::new(
                    PrintPathRole::SupportMaterial,
                    vec![
                        Point2::new(0.0, 0.0),
                        Point2::new(6.0, 0.0),
                        Point2::new(6.0, 6.0),
                        Point2::new(0.0, 6.0),
                    ],
                )
                .unwrap()
                .with_closed(true),
            ],
        )],
        options,
    )
    .unwrap()
}

fn first_support_material_delta(gcode: &str) -> f64 {
    let mut previous_e = 0.0;
    for line in gcode.lines() {
        let Some(rest) = line.strip_prefix(";EXTRUSION:print:") else {
            continue;
        };
        let Some((role_and_segment, e)) = rest.rsplit_once(':') else {
            continue;
        };
        let Ok(e) = e.parse::<f64>() else {
            continue;
        };
        if role_and_segment.starts_with("support_material:") {
            return e - previous_e;
        }
        previous_e = e;
    }
    panic!("missing support material extrusion");
}
