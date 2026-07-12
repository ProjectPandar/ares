use super::super::*;
use crate::WallSequence;
use serde_json::json;

#[test]
fn legacy_infill_first_wall_infill_order_sets_wall_sequence_and_infill_first() {
    for (legacy_value, expected_sequence) in [
        ("infill/inner wall/outer wall", WallSequence::InnerOuter),
        ("infill/outer wall/inner wall", WallSequence::OuterInner),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_infill_order": legacy_value })).unwrap();

        assert!(!options.values().contains_key("wall_infill_order"));
        assert!(options.is_infill_first().unwrap());
        assert_eq!(
            options.perimeter_options().unwrap().wall_sequence(),
            expected_sequence
        );
    }
}

#[test]
fn explicit_is_infill_first_overrides_legacy_wall_infill_order_default() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wall_infill_order": "infill/inner wall/outer wall",
        "is_infill_first": false
    }))
    .unwrap();

    assert!(!options.values().contains_key("wall_infill_order"));
    assert!(!options.is_infill_first().unwrap());
    assert_eq!(
        options.perimeter_options().unwrap().wall_sequence(),
        WallSequence::InnerOuter
    );
}
