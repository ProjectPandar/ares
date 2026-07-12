use super::*;
use crate::{SliceError, SliceOptions};
use serde_json::{Value, json};

fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000001,
        "actual {actual} expected {expected}"
    );
}

fn assert_support_widths(extrusion: &ExtrusionOptions, support: f64, interface: f64) {
    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterial),
        support,
    );
    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterialInterface),
        interface,
    );
}

fn support_interface_not_for_body_options(
    support_filament: Option<Value>,
    support_interface_filament: Option<Value>,
    not_for_body: bool,
    nozzle_diameter: Value,
    filament_diameter: Value,
) -> SliceOptions {
    let mut value = json!({
        "nozzle_diameter": nozzle_diameter,
        "filament_diameter": filament_diameter,
        "support_interface_not_for_body": not_for_body,
        "line_width": 0,
        "support_line_width": 0
    });
    let object = value.as_object_mut().unwrap();
    if let Some(support_filament) = support_filament {
        object.insert("support_filament".to_owned(), support_filament);
    }
    if let Some(support_interface_filament) = support_interface_filament {
        object.insert(
            "support_interface_filament".to_owned(),
            support_interface_filament,
        );
    }
    serde_json::from_value(value).unwrap()
}

fn two_selector_support_interface_not_for_body_options(
    support_filament: Option<Value>,
    support_interface_filament: Option<Value>,
    not_for_body: bool,
) -> SliceOptions {
    support_interface_not_for_body_options(
        support_filament,
        support_interface_filament,
        not_for_body,
        json!([0.4, 0.8]),
        json!([1.75, 2.85]),
    )
}

fn extrusion_delta(extrusion: &ExtrusionOptions, role: PrintPathRole) -> f64 {
    extrusion
        .extrusion_delta_for_segment(role, 0.2, false, 10.0)
        .unwrap()
}

#[test]
fn support_filament_changes_support_material_width_and_e_delta() {
    let first = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_support_hardware_for_tests(
            RoleHardwareValues::new(0.4, 1.75),
            RoleHardwareValues::new(0.4, 1.75),
        );
    let second = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_support_hardware_for_tests(
            RoleHardwareValues::new(0.8, 2.85),
            RoleHardwareValues::new(0.4, 1.75),
        );

    assert_approx_eq(second.width_for_role(PrintPathRole::SupportMaterial), 0.9);
    assert_approx_eq(
        first.width_for_role(PrintPathRole::SupportMaterialInterface),
        0.45,
    );
    assert!(
        second
            .extrusion_delta_for_segment(PrintPathRole::SupportMaterial, 0.2, false, 10.0)
            .unwrap()
            < first
                .extrusion_delta_for_segment(PrintPathRole::SupportMaterial, 0.2, false, 10.0)
                .unwrap()
    );
    assert_approx_eq(
        second
            .extrusion_delta_for_segment(PrintPathRole::SupportMaterialInterface, 0.2, false, 10.0)
            .unwrap(),
        first
            .extrusion_delta_for_segment(PrintPathRole::SupportMaterialInterface, 0.2, false, 10.0)
            .unwrap(),
    );
}

#[test]
fn support_interface_not_for_body_true_avoids_fixed_first_interface_selector_for_extrusion() {
    for support_filament in [None, Some(json!(0))] {
        let options = two_selector_support_interface_not_for_body_options(
            support_filament,
            Some(json!(1)),
            true,
        );

        let extrusion = options.extrusion_options().unwrap();

        assert_support_widths(&extrusion, 0.45, 0.45);
        assert!(
            extrusion_delta(&extrusion, PrintPathRole::SupportMaterial)
                < extrusion_delta(&extrusion, PrintPathRole::SupportMaterialInterface)
        );
    }
}

#[test]
fn support_interface_not_for_body_false_preserves_auto_support_selector() {
    let options =
        two_selector_support_interface_not_for_body_options(Some(json!(0)), Some(json!(1)), false);

    let extrusion = options.extrusion_options().unwrap();

    assert_support_widths(&extrusion, 0.45, 0.45);
    assert_approx_eq(
        extrusion_delta(&extrusion, PrintPathRole::SupportMaterial),
        extrusion_delta(&extrusion, PrintPathRole::SupportMaterialInterface),
    );
}

#[test]
fn support_interface_not_for_body_respects_positive_support_filament_precedence() {
    let options =
        two_selector_support_interface_not_for_body_options(Some(json!(2)), Some(json!(1)), true);

    let extrusion = options.extrusion_options().unwrap();

    assert_support_widths(&extrusion, 0.9, 0.45);
    assert!(
        extrusion_delta(&extrusion, PrintPathRole::SupportMaterial)
            < extrusion_delta(&extrusion, PrintPathRole::SupportMaterialInterface)
    );
}

#[test]
fn support_interface_not_for_body_fixed_non_first_interface_is_no_op_for_support_body() {
    let options =
        two_selector_support_interface_not_for_body_options(Some(json!(0)), Some(json!(2)), true);

    let extrusion = options.extrusion_options().unwrap();

    assert_support_widths(&extrusion, 0.45, 0.9);
    assert!(
        extrusion_delta(&extrusion, PrintPathRole::SupportMaterial)
            > extrusion_delta(&extrusion, PrintPathRole::SupportMaterialInterface)
    );
}

#[test]
fn support_interface_not_for_body_missing_or_zero_interface_is_no_op() {
    for support_interface_filament in [None, Some(json!(0))] {
        let options = two_selector_support_interface_not_for_body_options(
            Some(json!(0)),
            support_interface_filament,
            true,
        );

        let extrusion = options.extrusion_options().unwrap();

        assert_support_widths(&extrusion, 0.45, 0.45);
        assert_approx_eq(
            extrusion_delta(&extrusion, PrintPathRole::SupportMaterial),
            extrusion_delta(&extrusion, PrintPathRole::SupportMaterialInterface),
        );
    }
}

#[test]
fn support_interface_not_for_body_single_selector_falls_back_to_first() {
    let options = support_interface_not_for_body_options(
        Some(json!(0)),
        Some(json!(1)),
        true,
        json!([0.4]),
        json!([1.75]),
    );

    let extrusion = options.extrusion_options().unwrap();

    assert_support_widths(&extrusion, 0.45, 0.45);
    assert_approx_eq(
        extrusion_delta(&extrusion, PrintPathRole::SupportMaterial),
        extrusion_delta(&extrusion, PrintPathRole::SupportMaterialInterface),
    );
}

#[test]
fn slice_options_extrusion_options_reject_invalid_support_interface_not_for_body() {
    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "support_interface_not_for_body": value })).unwrap();

        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn support_interface_filament_changes_interface_width_and_e_delta() {
    let first = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_support_hardware_for_tests(
            RoleHardwareValues::new(0.4, 1.75),
            RoleHardwareValues::new(0.4, 1.75),
        );
    let second = ExtrusionOptions::new_for_tests(0.4, 1.75, 0.0, (0.0, 0.0), 0.0)
        .with_support_hardware_for_tests(
            RoleHardwareValues::new(0.4, 1.75),
            RoleHardwareValues::new(0.8, 2.85),
        );

    assert_approx_eq(second.width_for_role(PrintPathRole::SupportMaterial), 0.45);
    assert_approx_eq(
        second.width_for_role(PrintPathRole::SupportMaterialInterface),
        0.9,
    );
    assert_approx_eq(
        second
            .extrusion_delta_for_segment(PrintPathRole::SupportMaterial, 0.2, false, 10.0)
            .unwrap(),
        first
            .extrusion_delta_for_segment(PrintPathRole::SupportMaterial, 0.2, false, 10.0)
            .unwrap(),
    );
    assert!(
        second
            .extrusion_delta_for_segment(PrintPathRole::SupportMaterialInterface, 0.2, false, 10.0,)
            .unwrap()
            < first
                .extrusion_delta_for_segment(
                    PrintPathRole::SupportMaterialInterface,
                    0.2,
                    false,
                    10.0,
                )
                .unwrap()
    );
}

#[test]
fn slice_options_missing_support_filaments_default_to_first_hardware() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "line_width": 0,
        "support_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterial),
        0.45,
    );
    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterialInterface),
        0.45,
    );
}

#[test]
fn slice_options_support_filaments_accept_explicit_zero() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "support_filament": 0,
        "support_interface_filament": 0,
        "line_width": 0,
        "support_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterial),
        0.45,
    );
    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterialInterface),
        0.45,
    );
}

#[test]
fn slice_options_support_filaments_accept_numeric_strings_and_float_integers() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "support_filament": "2",
        "support_interface_filament": 2.0,
        "line_width": 0,
        "support_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterial),
        0.9,
    );
    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterialInterface),
        0.9,
    );
}

#[test]
fn slice_options_reject_invalid_explicit_support_filaments() {
    for (key, value) in [
        ("support_filament", json!(-1)),
        ("support_filament", json!(1.5)),
        ("support_interface_filament", json!("2.5")),
        ("support_interface_filament", json!("fast")),
        ("support_interface_filament", json!(true)),
        ("support_filament", json!([])),
        ("support_interface_filament", json!({})),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn slice_options_support_selector_fallback_is_independent_per_hardware_vector() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [1.75, 2.85],
        "support_filament": 2,
        "support_interface_filament": u64::MAX,
        "line_width": 0,
        "support_line_width": 0
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();

    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterial),
        0.45,
    );
    assert_approx_eq(
        extrusion.width_for_role(PrintPathRole::SupportMaterialInterface),
        0.45,
    );
    assert!(
        extrusion
            .extrusion_delta_for_segment(PrintPathRole::SupportMaterial, 0.2, false, 10.0)
            .unwrap()
            < extrusion
                .extrusion_delta_for_segment(
                    PrintPathRole::SupportMaterialInterface,
                    0.2,
                    false,
                    10.0,
                )
                .unwrap()
    );
}
