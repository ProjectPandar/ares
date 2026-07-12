use super::super::super::*;
use crate::{BrimType, SliceError};
use serde_json::json;

#[test]
fn brim_ears_max_angle_defaults_to_orca_value() {
    let brim = SliceOptions::default().brim_options().unwrap();

    assert_eq!(brim.brim_type(), BrimType::AutoBrim);
    assert_eq!(brim.brim_ears_max_angle_degrees(), 125.0);
    assert_eq!(brim.brim_ears_detection_length_mm(), 1.0);
}

#[test]
fn combine_brims_defaults_to_false_and_parses_bool() {
    assert!(
        !SliceOptions::default()
            .brim_options()
            .unwrap()
            .combine_brims()
    );

    let options: SliceOptions = serde_json::from_value(json!({ "combine_brims": true })).unwrap();
    assert!(options.brim_options().unwrap().combine_brims());
}

#[test]
fn rejects_non_bool_combine_brims() {
    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "combine_brims": value })).unwrap();
        let err = options.brim_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("combine_brims"));
    }
}

#[test]
fn parses_brim_ears_max_angle_boundaries() {
    for (value, expected) in [(json!(0), 0.0), (json!(180), 180.0), (json!("45.5"), 45.5)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "brim_ears_max_angle": value })).unwrap();

        assert_eq!(
            options
                .brim_options()
                .unwrap()
                .brim_ears_max_angle_degrees(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_brim_ears_max_angle_values() {
    for value in [
        json!(-0.1),
        json!(180.1),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("Infinity"),
        json!(true),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "brim_ears_max_angle": value })).unwrap();
        let err = options.brim_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("brim_ears_max_angle"));
    }
}

#[test]
fn parses_brim_ears_detection_length_boundaries() {
    for (value, expected) in [(json!(0), 0.0), (json!("2.5"), 2.5)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "brim_ears_detection_length": value })).unwrap();

        assert_eq!(
            options
                .brim_options()
                .unwrap()
                .brim_ears_detection_length_mm(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_brim_ears_detection_length_values() {
    for value in [
        json!(-0.1),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("Infinity"),
        json!(true),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "brim_ears_detection_length": value })).unwrap();
        let err = options.brim_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("brim_ears_detection_length"));
    }
}

#[test]
fn active_brim_efc_outline_gate_reaches_brim_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "brim_use_efc_outline": true,
        "elefant_foot_compensation": 0.2,
        "elefant_foot_compensation_layers": 1,
        "raft_layers": 0
    }))
    .unwrap();

    assert_eq!(
        options.brim_options().unwrap().efc_outline_offset_mm(),
        Some(0.2)
    );
}

#[test]
fn brim_efc_outline_gate_requires_option_compensation_layers_and_no_raft() {
    for extra in [
        json!({}),
        json!({
            "brim_use_efc_outline": false,
            "elefant_foot_compensation": 0.2,
            "elefant_foot_compensation_layers": 1,
            "raft_layers": 0
        }),
        json!({
            "brim_use_efc_outline": true,
            "elefant_foot_compensation": 0.0,
            "elefant_foot_compensation_layers": 1,
            "raft_layers": 0
        }),
        json!({
            "brim_use_efc_outline": true,
            "elefant_foot_compensation": 0.2,
            "elefant_foot_compensation_layers": 0,
            "raft_layers": 0
        }),
        json!({
            "brim_use_efc_outline": true,
            "elefant_foot_compensation": 0.2,
            "elefant_foot_compensation_layers": 1,
            "raft_layers": 1
        }),
    ] {
        let options: SliceOptions = serde_json::from_value(extra).unwrap();

        assert_eq!(
            options.brim_options().unwrap().efc_outline_offset_mm(),
            None
        );
    }
}

#[test]
fn rejects_invalid_brim_efc_outline_gate_options() {
    for extra in [
        json!({ "brim_use_efc_outline": "true" }),
        json!({ "elefant_foot_compensation": -0.1 }),
        json!({ "elefant_foot_compensation": "bad" }),
        json!({ "elefant_foot_compensation_layers": -1 }),
        json!({ "elefant_foot_compensation_layers": 1.5 }),
        json!({ "raft_layers": -1 }),
        json!({ "raft_layers": 1.5 }),
        json!({ "raft_layers": 101 }),
        json!({ "raft_layers": "101" }),
    ] {
        let options: SliceOptions = serde_json::from_value(extra).unwrap();

        assert!(options.brim_options().is_err());
    }
}
