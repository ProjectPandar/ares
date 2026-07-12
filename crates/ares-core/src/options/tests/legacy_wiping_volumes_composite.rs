use super::super::*;
use serde_json::json;

#[test]
fn infers_default_wiping_volume_matrix_as_not_custom() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wiping_volumes_matrix": [0, 140, 140, 0],
        "future_orca_key": "preserved"
    }))
    .unwrap();

    assert_eq!(
        options.values()["wiping_volumes_matrix"],
        json!([0, 140, 140, 0])
    );
    assert_eq!(
        options.values()["wiping_volumes_use_custom_matrix"],
        json!(false)
    );
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn infers_off_diagonal_non_default_wiping_volume_matrix_as_custom() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wiping_volumes_matrix": [0, 140, 141, 0]
    }))
    .unwrap();

    assert_eq!(
        options.values()["wiping_volumes_use_custom_matrix"],
        json!(true)
    );
}

#[test]
fn treats_approximately_default_off_diagonal_wiping_volume_as_default() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wiping_volumes_matrix": [0, 140.00005, 140, 0]
    }))
    .unwrap();

    assert_eq!(
        options.values()["wiping_volumes_use_custom_matrix"],
        json!(false)
    );
}

#[test]
fn ignores_diagonal_values_when_inferring_wiping_volume_custom_flag() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wiping_volumes_matrix": [5, 140, 140, 7]
    }))
    .unwrap();

    assert_eq!(
        options.values()["wiping_volumes_use_custom_matrix"],
        json!(false)
    );
}

#[test]
fn preserves_existing_wiping_volume_custom_flag() {
    for existing in [json!(true), json!(false), json!("legacy")] {
        let options: SliceOptions = serde_json::from_value(json!({
            "wiping_volumes_matrix": [0, 140, 141, 0],
            "wiping_volumes_use_custom_matrix": existing.clone()
        }))
        .unwrap();

        assert_eq!(
            options.values()["wiping_volumes_use_custom_matrix"],
            existing
        );
    }
}

#[test]
fn parses_wiping_volume_matrix_numeric_string_and_scalar_forms() {
    let string_matrix: SliceOptions = serde_json::from_value(json!({
        "wiping_volumes_matrix": "0;140;140;0"
    }))
    .unwrap();
    assert_eq!(
        string_matrix.values()["wiping_volumes_use_custom_matrix"],
        json!(false)
    );

    let scalar_matrix: SliceOptions = serde_json::from_value(json!({
        "wiping_volumes_matrix": 0
    }))
    .unwrap();
    assert_eq!(
        scalar_matrix.values()["wiping_volumes_use_custom_matrix"],
        json!(false)
    );
}

#[test]
fn rejects_invalid_wiping_volume_matrix_values() {
    for value in [
        json!([]),
        json!(""),
        json!("0;bad"),
        json!({"matrix": []}),
        json!(true),
    ] {
        let error = serde_json::from_value::<SliceOptions>(json!({
            "wiping_volumes_matrix": value
        }))
        .unwrap_err();

        assert!(error.to_string().contains("wiping_volumes_matrix must"));
    }
}
