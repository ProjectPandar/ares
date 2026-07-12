use serde_json::json;

use super::super::validate_non_diff_stride2_float_vectors;
use crate::SliceError;

#[test]
fn stride2_float_type_check_accepts_numeric_source_and_target_arrays() {
    assert_eq!(
        validate_non_diff_stride2_float_vectors(
            "machine_max_speed_x",
            &json!([10.0, 20.0]),
            &json!([30.0, 40.0]),
        ),
        Ok(())
    );
}

#[test]
fn stride2_float_type_check_accepts_empty_arrays() {
    assert_eq!(
        validate_non_diff_stride2_float_vectors("machine_max_speed_x", &json!([]), &json!([])),
        Ok(())
    );
}

#[test]
fn stride2_float_type_check_rejects_non_array_source() {
    let error = validate_non_diff_stride2_float_vectors(
        "machine_max_speed_x",
        &json!(10.0),
        &json!([30.0, 40.0]),
    )
    .unwrap_err();

    assert_stride2_float_error(error, "machine_max_speed_x");
}

#[test]
fn stride2_float_type_check_rejects_non_array_target() {
    let error = validate_non_diff_stride2_float_vectors(
        "machine_max_speed_x",
        &json!([10.0, 20.0]),
        &json!(30.0),
    )
    .unwrap_err();

    assert_stride2_float_error(error, "machine_max_speed_x");
}

#[test]
fn stride2_float_type_check_rejects_non_numeric_source_entries() {
    let error = validate_non_diff_stride2_float_vectors(
        "machine_max_speed_x",
        &json!([10.0, "fast"]),
        &json!([30.0, 40.0]),
    )
    .unwrap_err();

    assert_stride2_float_error(error, "machine_max_speed_x");
}

#[test]
fn stride2_float_type_check_rejects_non_numeric_target_entries() {
    let error = validate_non_diff_stride2_float_vectors(
        "machine_max_speed_x",
        &json!([10.0, 20.0]),
        &json!([30.0, null]),
    )
    .unwrap_err();

    assert_stride2_float_error(error, "machine_max_speed_x");
}

fn assert_stride2_float_error(error: SliceError, key: &str) {
    let SliceError::InvalidInput(message) = error else {
        panic!("expected InvalidInput error");
    };
    assert!(message.contains(key));
    assert!(message.contains("ConfigOptionFloats for stride=2"));
}
