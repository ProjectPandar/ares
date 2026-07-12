use super::*;
use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, gcode::format_gcode,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn pipeline_rejects_invalid_support_enable_before_model_loading() {
    for value in [json!("true"), json!("false"), json!(1), Value::Null] {
        let options = options(json!({ "enable_support": value }));
        let err = run_slicing_pipeline(b"not a model", &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("enable_support"));
    }
}

#[test]
fn valid_support_enable_values_preserve_current_pipeline_artifacts() {
    let baseline_options = options(json!({ "enable_support": false }));
    let baseline = run_slicing_pipeline(square_pyramid_ascii_stl(), &baseline_options).unwrap();
    let baseline_gcode =
        String::from_utf8(format_gcode(&baseline, &baseline_options).unwrap()).unwrap();

    let enabled_options = options(json!({ "enable_support": true }));
    let enabled = run_slicing_pipeline(square_pyramid_ascii_stl(), &enabled_options).unwrap();
    let enabled_gcode =
        String::from_utf8(format_gcode(&enabled, &enabled_options).unwrap()).unwrap();

    assert_eq!(enabled.layer_print_paths(), baseline.layer_print_paths());
    assert_eq!(
        enabled.layer_toolpath_moves(),
        baseline.layer_toolpath_moves()
    );
    assert_eq!(
        enabled.layer_extrusion_moves(),
        baseline.layer_extrusion_moves()
    );
    assert_eq!(enabled.layer_speed_moves(), baseline.layer_speed_moves());
    assert_eq!(enabled.diagnostics(), baseline.diagnostics());
    assert_eq!(enabled_gcode, baseline_gcode);
}

#[test]
fn disabled_or_omitted_support_removes_proxy_artifacts_from_pipeline() {
    for role in [
        PrintPathRole::SupportMaterial,
        PrintPathRole::SupportMaterialInterface,
    ] {
        for extra in [json!({}), json!({ "enable_support": false })] {
            let options = options(extra);
            let pipeline = single_path_pipeline(&options, role, 1);
            let output = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

            assert!(pipeline.layer_print_paths()[1].paths().is_empty());
            assert!(pipeline.layer_toolpath_moves()[1].moves().is_empty());
            assert!(pipeline.layer_extrusion_moves()[1].moves().is_empty());
            assert!(pipeline.layer_speed_moves()[1].moves().is_empty());
            assert_eq!(pipeline.diagnostics().total_print_path_count(), 0);
            assert_no_support_proxy_gcode(&output);
        }
    }
}

#[test]
fn enforced_support_layers_preserves_proxy_artifacts_without_enable_support() {
    for (role, marker) in [
        (PrintPathRole::SupportMaterial, "support_material"),
        (
            PrintPathRole::SupportMaterialInterface,
            "support_material_interface",
        ),
    ] {
        for extra in [
            json!({ "enforce_support_layers": 1 }),
            json!({ "enable_support": false, "enforce_support_layers": 1 }),
        ] {
            let options = options(extra);
            let pipeline = single_path_pipeline(&options, role, 1);
            let output = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

            assert_eq!(pipeline.layer_print_paths()[1].paths().len(), 1);
            assert!(!pipeline.layer_toolpath_moves()[1].moves().is_empty());
            assert!(!pipeline.layer_extrusion_moves()[1].moves().is_empty());
            assert!(!pipeline.layer_speed_moves()[1].moves().is_empty());
            assert_eq!(pipeline.diagnostics().total_print_path_count(), 1);
            assert!(output.contains(&format!(";PRINT_PATH:{marker}:")));
            assert!(output.contains(&format!(";SPEED:print:{marker}:")));
            assert!(output.contains(&format!(";EXTRUSION:print:{marker}:")));
            assert!(output.contains(&format!(";MOVE:print:{marker}:")));
        }
    }
}

#[test]
fn zero_enforced_support_layers_keeps_disabled_support_filter() {
    for role in [
        PrintPathRole::SupportMaterial,
        PrintPathRole::SupportMaterialInterface,
    ] {
        for extra in [
            json!({ "enforce_support_layers": 0 }),
            json!({ "enable_support": false, "enforce_support_layers": 0 }),
            json!({ "enforce_support_layers": "0" }),
            json!({ "enable_support": false, "enforce_support_layers": "0" }),
        ] {
            let options = options(extra);
            let pipeline = single_path_pipeline(&options, role, 1);
            let output = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

            assert!(pipeline.layer_print_paths()[1].paths().is_empty());
            assert!(pipeline.layer_toolpath_moves()[1].moves().is_empty());
            assert!(pipeline.layer_extrusion_moves()[1].moves().is_empty());
            assert!(pipeline.layer_speed_moves()[1].moves().is_empty());
            assert_eq!(pipeline.diagnostics().total_print_path_count(), 0);
            assert_no_support_proxy_gcode(&output);
        }
    }
}

#[test]
fn raft_layers_preserves_proxy_artifacts_without_enable_support() {
    for (role, marker) in [
        (PrintPathRole::SupportMaterial, "support_material"),
        (
            PrintPathRole::SupportMaterialInterface,
            "support_material_interface",
        ),
    ] {
        for extra in [
            json!({ "raft_layers": 1 }),
            json!({ "enable_support": false, "raft_layers": 1 }),
        ] {
            let options = options(extra);
            let pipeline = single_path_pipeline(&options, role, 1);
            let output = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

            assert_eq!(pipeline.layer_print_paths()[1].paths().len(), 1);
            assert!(!pipeline.layer_toolpath_moves()[1].moves().is_empty());
            assert!(!pipeline.layer_extrusion_moves()[1].moves().is_empty());
            assert!(!pipeline.layer_speed_moves()[1].moves().is_empty());
            assert_eq!(pipeline.diagnostics().total_print_path_count(), 1);
            assert!(output.contains(&format!(";PRINT_PATH:{marker}:")));
            assert!(output.contains(&format!(";SPEED:print:{marker}:")));
            assert!(output.contains(&format!(";EXTRUSION:print:{marker}:")));
            assert!(output.contains(&format!(";MOVE:print:{marker}:")));
        }
    }
}

#[test]
fn zero_raft_layers_keeps_disabled_support_filter() {
    for role in [
        PrintPathRole::SupportMaterial,
        PrintPathRole::SupportMaterialInterface,
    ] {
        for extra in [
            json!({ "raft_layers": 0 }),
            json!({ "enable_support": false, "raft_layers": 0 }),
            json!({ "raft_layers": "0" }),
            json!({ "enable_support": false, "raft_layers": "0" }),
        ] {
            let options = options(extra);
            let pipeline = single_path_pipeline(&options, role, 1);
            let output = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

            assert!(pipeline.layer_print_paths()[1].paths().is_empty());
            assert!(pipeline.layer_toolpath_moves()[1].moves().is_empty());
            assert!(pipeline.layer_extrusion_moves()[1].moves().is_empty());
            assert!(pipeline.layer_speed_moves()[1].moves().is_empty());
            assert_eq!(pipeline.diagnostics().total_print_path_count(), 0);
            assert_no_support_proxy_gcode(&output);
        }
    }
}

#[test]
fn disabled_support_removes_support_interface_ironing_paths() {
    let options = options(json!({
        "enable_support": false,
        "support_ironing": true,
        "support_ironing_spacing": 0.5
    }));
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterialInterface)],
        )],
        &options,
    )
    .unwrap();

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn enforced_support_layers_preserves_support_interface_ironing_paths() {
    let options = options(json!({
        "enable_support": false,
        "enforce_support_layers": 1,
        "support_ironing": true,
        "support_ironing_spacing": 0.5
    }));
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterialInterface)],
        )],
        &options,
    )
    .unwrap();

    let roles = finalized[0]
        .paths()
        .iter()
        .map(|path| (path.role(), path.extrusion_role()))
        .collect::<Vec<_>>();

    assert!(roles.contains(&(PrintPathRole::SupportMaterialInterface, None)));
    assert!(roles.contains(&(
        PrintPathRole::Ironing,
        Some(PrintPathRole::SupportMaterialInterface)
    )));
}

#[test]
fn raft_layers_preserves_support_interface_ironing_paths() {
    let options = options(json!({
        "enable_support": false,
        "raft_layers": 1,
        "support_ironing": true,
        "support_ironing_spacing": 0.5
    }));
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterialInterface)],
        )],
        &options,
    )
    .unwrap();

    let roles = finalized[0]
        .paths()
        .iter()
        .map(|path| (path.role(), path.extrusion_role()))
        .collect::<Vec<_>>();

    assert!(roles.contains(&(PrintPathRole::SupportMaterialInterface, None)));
    assert!(roles.contains(&(
        PrintPathRole::Ironing,
        Some(PrintPathRole::SupportMaterialInterface)
    )));
}

#[test]
fn enabled_support_preserves_current_proxy_roles() {
    let options = options(json!({
        "enable_support": true,
        "support_ironing": true,
        "support_ironing_spacing": 0.5
    }));
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![
                support_line(PrintPathRole::SupportMaterial),
                support_rectangle(PrintPathRole::SupportMaterialInterface),
            ],
        )],
        &options,
    )
    .unwrap();

    let roles = finalized[0]
        .paths()
        .iter()
        .map(|path| (path.role(), path.extrusion_role()))
        .collect::<Vec<_>>();

    assert!(roles.contains(&(PrintPathRole::SupportMaterial, None)));
    assert!(roles.contains(&(PrintPathRole::SupportMaterialInterface, None)));
    assert!(roles.contains(&(
        PrintPathRole::Ironing,
        Some(PrintPathRole::SupportMaterialInterface)
    )));
}

#[test]
fn disabled_support_still_validates_support_options_before_filtering() {
    for (key, value) in [
        ("support_interface_spacing", json!("fast")),
        ("support_top_z_distance", json!(-0.1)),
        ("enforce_support_layers", json!(5001)),
        ("raft_layers", json!(101)),
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(
                0,
                0.2,
                vec![support_rectangle(PrintPathRole::SupportMaterialInterface)],
            )],
            &options(json!({
                "enable_support": false,
                key: value
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn disabled_support_preserves_ordinary_ironing_paths() {
    let options = options(json!({ "enable_support": false }));
    let ironing = support_line(PrintPathRole::Ironing);
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(0, 0.2, vec![ironing.clone()])],
        &options,
    )
    .unwrap();

    assert_eq!(finalized[0].paths(), [ironing]);
}

fn support_line(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, vec![Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)]).unwrap()
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(
        role,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true)
}

fn assert_no_support_proxy_gcode(output: &str) {
    for role in [
        "support_material",
        "support_material_interface",
        "support_ironing",
    ] {
        assert!(!output.contains(&format!(";PRINT_PATH:{role}:")));
        assert!(!output.contains(&format!(";SPEED:print:{role}:")));
        assert!(!output.contains(&format!(";EXTRUSION:print:{role}:")));
        assert!(!output.contains(&format!(";MOVE:print:{role}:")));
    }
}

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}
