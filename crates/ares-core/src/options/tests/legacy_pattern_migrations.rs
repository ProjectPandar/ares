use super::super::*;
use serde_json::json;

#[test]
fn preserves_zig_zag_pattern_values_without_migration() {
    // ZigZag keeps its own enum semantics; patterns are no longer rewritten
    // to rectilinear (`PrintConfig.cpp` `ipZigZag`).
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_pattern": "zig-zag",
        "top_surface_pattern": "zig-zag",
        "bottom_surface_pattern": "zig-zag",
        "internal_solid_infill_pattern": "zig-zag",
        "ironing_pattern": "zig-zag",
        "future_orca_key": "preserved"
    }))
    .unwrap();

    for key in [
        "sparse_infill_pattern",
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
    ] {
        assert_eq!(options.values()[key], json!("zig-zag"));
    }
    assert_eq!(options.values()["ironing_pattern"], json!("zig-zag"));
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn preserves_non_matching_legacy_pattern_strings() {
    for value in ["rectilinear", "Zig-Zag", "zig-zag-grid", "grid"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "sparse_infill_pattern": value,
            "top_surface_pattern": value,
            "bottom_surface_pattern": value,
            "internal_solid_infill_pattern": value,
            "ironing_pattern": value,
            "support_ironing_pattern": value
        }))
        .unwrap();

        for key in [
            "sparse_infill_pattern",
            "top_surface_pattern",
            "bottom_surface_pattern",
            "internal_solid_infill_pattern",
            "ironing_pattern",
            "support_ironing_pattern",
        ] {
            assert_eq!(options.values()[key], json!(value));
        }
    }
}

#[test]
fn preserves_non_string_legacy_pattern_values() {
    for value in [
        json!(true),
        json!(3),
        json!(null),
        json!(["zig-zag"]),
        json!({"pattern": "zig-zag"}),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "sparse_infill_pattern": value.clone(),
            "top_surface_pattern": value.clone(),
            "bottom_surface_pattern": value.clone(),
            "internal_solid_infill_pattern": value.clone(),
            "ironing_pattern": value.clone(),
            "support_ironing_pattern": value.clone()
        }))
        .unwrap();

        for key in [
            "sparse_infill_pattern",
            "top_surface_pattern",
            "bottom_surface_pattern",
            "internal_solid_infill_pattern",
            "ironing_pattern",
            "support_ironing_pattern",
        ] {
            assert_eq!(options.values()[key], value);
        }
    }
}
