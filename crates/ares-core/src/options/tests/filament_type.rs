use crate::{FilamentTypeDisplay, SliceError, SliceOptions};
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn display(value: &str, displayed: &str) -> FilamentTypeDisplay {
    FilamentTypeDisplay {
        value: value.to_owned(),
        displayed: displayed.to_owned(),
    }
}

#[test]
fn missing_filament_type_returns_empty_display() {
    assert_eq!(
        options(json!({})).filament_type_display(0).unwrap(),
        display("", "")
    );
}

#[test]
fn missing_support_flag_returns_raw_filament_type() {
    assert_eq!(
        options(json!({ "filament_type": ["PLA", "PETG"] }))
            .filament_type_display(1)
            .unwrap(),
        display("PETG", "PETG")
    );
}

#[test]
fn non_support_filament_returns_raw_filament_type() {
    assert_eq!(
        options(json!({
            "filament_type": ["PLA", "PETG"],
            "filament_is_support": [false, false]
        }))
        .filament_type_display(1)
        .unwrap(),
        display("PETG", "PETG")
    );
}

#[test]
fn support_filament_ids_map_to_support_display_names() {
    let options = options(json!({
        "filament_type": ["PLA", "PA"],
        "filament_is_support": [true, true],
        "filament_id": ["GFS00", "GFS01"]
    }));

    assert_eq!(
        options.filament_type_display(0).unwrap(),
        display("PLA-S", "Sup.PLA")
    );
    assert_eq!(
        options.filament_type_display(1).unwrap(),
        display("PA-S", "Sup.PA")
    );
}

#[test]
fn support_filament_type_fallback_maps_pla_and_pa() {
    let options = options(json!({
        "filament_type": ["PLA", "PA"],
        "filament_is_support": [true, true]
    }));

    assert_eq!(
        options.filament_type_display(0).unwrap(),
        display("PLA-S", "Sup.PLA")
    );
    assert_eq!(
        options.filament_type_display(1).unwrap(),
        display("PA-S", "Sup.PA")
    );
}

#[test]
fn support_unknown_material_passes_through() {
    assert_eq!(
        options(json!({
            "filament_type": ["PETG"],
            "filament_is_support": [true]
        }))
        .filament_type_display(0)
        .unwrap(),
        display("PETG", "PETG")
    );
}

#[test]
fn vector_get_at_uses_first_value_for_out_of_range_id() {
    assert_eq!(
        options(json!({
            "filament_type": ["PLA"],
            "filament_is_support": [true]
        }))
        .filament_type_display(9)
        .unwrap(),
        display("PLA-S", "Sup.PLA")
    );
}

#[test]
fn invalid_filament_type_boundary_values_return_invalid_input() {
    for value in [
        json!({ "filament_type": "PLA" }),
        json!({ "filament_type": [] }),
        json!({ "filament_type": [7] }),
        json!({ "filament_type": ["PLA"], "filament_is_support": true }),
        json!({ "filament_type": ["PLA"], "filament_is_support": [] }),
        json!({ "filament_type": ["PLA"], "filament_is_support": ["true"] }),
        json!({ "filament_type": ["PLA"], "filament_is_support": [true], "filament_id": "GFS00" }),
        json!({ "filament_type": ["PLA"], "filament_is_support": [true], "filament_id": [] }),
        json!({ "filament_type": ["PLA"], "filament_is_support": [true], "filament_id": [7] }),
    ] {
        assert!(matches!(
            options(value).filament_type_display(0),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn validation_api_remains_callable_after_filament_type_api() {
    let options = options(json!({ "layer_height": 0.0 }));
    let errors = options.validate_print_config(true).unwrap();

    assert_eq!(errors["layer_height"], "invalid value 0");
}

#[test]
fn filament_identity_queries_return_first_string_entries() {
    let options = options(json!({
        "filament_vendor": ["Orca", "Ignored"],
        "filament_type": ["PLA", "PETG"]
    }));

    assert_eq!(options.filament_vendor().unwrap(), "Orca");
    assert_eq!(options.filament_type().unwrap(), "PLA");
}

#[test]
fn filament_identity_queries_return_empty_for_missing_and_empty_vectors() {
    let missing = options(json!({}));
    let empty = options(json!({
        "filament_vendor": [],
        "filament_type": []
    }));

    assert_eq!(missing.filament_vendor().unwrap(), "");
    assert_eq!(missing.filament_type().unwrap(), "");
    assert_eq!(empty.filament_vendor().unwrap(), "");
    assert_eq!(empty.filament_type().unwrap(), "");
}

#[test]
fn filament_identity_queries_ignore_later_entries() {
    let options = options(json!({
        "filament_vendor": ["FirstVendor", 7],
        "filament_type": ["FirstType", false]
    }));

    assert_eq!(options.filament_vendor().unwrap(), "FirstVendor");
    assert_eq!(options.filament_type().unwrap(), "FirstType");
}

#[test]
fn invalid_filament_identity_query_values_return_invalid_input() {
    for value in [
        json!({ "filament_vendor": "Orca" }),
        json!({ "filament_vendor": [7] }),
        json!({ "filament_type": "PLA" }),
        json!({ "filament_type": [false] }),
    ] {
        let options = options(value);
        let result = options
            .filament_vendor()
            .and_then(|_| options.filament_type());

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    }
}
