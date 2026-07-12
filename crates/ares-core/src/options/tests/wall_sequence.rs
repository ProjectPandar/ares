use super::super::*;
use crate::WallSequence;
use serde_json::json;

#[test]
fn wall_sequence_defaults_to_inner_outer() {
    let options = SliceOptions::default();

    assert_eq!(
        options.perimeter_options().unwrap().wall_sequence(),
        WallSequence::InnerOuter
    );
}

#[test]
fn parses_supported_wall_sequence_values() {
    for (value, expected) in [
        ("inner wall/outer wall", WallSequence::InnerOuter),
        ("outer wall/inner wall", WallSequence::OuterInner),
        ("inner-outer-inner wall", WallSequence::InnerOuterInner),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_sequence": value })).unwrap();

        assert_eq!(
            options.perimeter_options().unwrap().wall_sequence(),
            expected
        );
    }
}

#[test]
fn legacy_wall_infill_order_values_parse_as_wall_sequence() {
    for (value, expected) in [
        ("inner wall/outer wall/infill", WallSequence::InnerOuter),
        ("infill/inner wall/outer wall", WallSequence::InnerOuter),
        ("outer wall/inner wall/infill", WallSequence::OuterInner),
        ("infill/outer wall/inner wall", WallSequence::OuterInner),
        (
            "inner-outer-inner wall/infill",
            WallSequence::InnerOuterInner,
        ),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_infill_order": value })).unwrap();

        assert_eq!(
            options.perimeter_options().unwrap().wall_sequence(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_wall_sequence_values() {
    for value in [json!("outer-inner"), json!(true), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_sequence": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn only_one_wall_first_layer_defaults_to_false() {
    let options = SliceOptions::default();

    assert!(!options
        .perimeter_options()
        .unwrap()
        .only_one_wall_first_layer());
}

#[test]
fn parses_only_one_wall_first_layer() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "only_one_wall_first_layer": true })).unwrap();

    assert!(options
        .perimeter_options()
        .unwrap()
        .only_one_wall_first_layer());
}

#[test]
fn rejects_invalid_only_one_wall_first_layer_values() {
    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "only_one_wall_first_layer": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("only_one_wall_first_layer must be a boolean")
        ));
    }
}

#[test]
fn only_one_wall_top_defaults_to_false() {
    let options = SliceOptions::default();

    assert!(!options.perimeter_options().unwrap().only_one_wall_top());
}

#[test]
fn parses_only_one_wall_top() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "only_one_wall_top": true })).unwrap();

    assert!(options.perimeter_options().unwrap().only_one_wall_top());
}

#[test]
fn rejects_invalid_only_one_wall_top_values() {
    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "only_one_wall_top": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("only_one_wall_top must be a boolean")
        ));
    }
}

#[test]
fn legacy_top_one_wall_type_enables_only_one_wall_top_perimeter_option() {
    let options: SliceOptions = serde_json::from_value(json!({
        "top_one_wall_type": "top"
    }))
    .unwrap();

    assert_eq!(options.values()["only_one_wall_top"], json!(true));
    assert!(options.perimeter_options().unwrap().only_one_wall_top());
}

#[test]
fn alternate_extra_wall_defaults_to_false() {
    let options = SliceOptions::default();

    assert!(!options.perimeter_options().unwrap().alternate_extra_wall());
}

#[test]
fn parses_alternate_extra_wall_and_sparse_infill_density_for_perimeters() {
    let options: SliceOptions = serde_json::from_value(json!({
        "alternate_extra_wall": true,
        "sparse_infill_density": 35
    }))
    .unwrap();
    let perimeters = options.perimeter_options().unwrap();

    assert!(perimeters.alternate_extra_wall());
    assert_eq!(perimeters.sparse_infill_density_percent(), 35.0);
}

#[test]
fn precise_outer_wall_defaults_to_true_and_uses_layer_height() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.28
    }))
    .unwrap();
    let perimeters = options.perimeter_options().unwrap();

    assert!(perimeters.precise_outer_wall());
    assert_eq!(perimeters.layer_height_mm(), 0.28);
}

#[test]
fn parses_precise_outer_wall_boolean() {
    for (value, expected) in [(true, true), (false, false)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "precise_outer_wall": value })).unwrap();

        let perimeters = options.perimeter_options().unwrap();
        assert_eq!(perimeters.precise_outer_wall(), expected);
    }
}

#[test]
fn rejects_invalid_precise_outer_wall_values() {
    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "precise_outer_wall": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("precise_outer_wall must be a boolean")
        ));
    }
}

#[test]
fn rejects_invalid_alternate_extra_wall_values() {
    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "alternate_extra_wall": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("alternate_extra_wall must be a boolean")
        ));
    }
}
