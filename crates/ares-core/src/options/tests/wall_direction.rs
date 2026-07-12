use super::super::*;
use crate::WallDirection;
use serde_json::json;

#[test]
fn wall_direction_defaults_to_counter_clockwise() {
    let options = SliceOptions::default();

    assert_eq!(
        options.perimeter_options().unwrap().wall_direction(),
        WallDirection::CounterClockwise
    );
}

#[test]
fn parses_supported_wall_direction_values() {
    for (value, expected) in [
        ("ccw", WallDirection::CounterClockwise),
        ("cw", WallDirection::Clockwise),
        ("auto", WallDirection::CounterClockwise),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_direction": value })).unwrap();

        assert_eq!(
            options.perimeter_options().unwrap().wall_direction(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_wall_direction_values() {
    for value in [json!("clockwise"), json!(true), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_direction": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
