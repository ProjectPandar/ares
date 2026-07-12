use super::super::*;
use crate::WallGenerator;
use serde_json::json;

#[test]
fn wall_generator_defaults_to_arachne() {
    let options = SliceOptions::default();

    assert_eq!(
        options.perimeter_options().unwrap().wall_generator(),
        WallGenerator::Arachne
    );
}

#[test]
fn parses_supported_wall_generator_values() {
    for (value, expected) in [
        ("classic", WallGenerator::Classic),
        ("arachne", WallGenerator::Arachne),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_generator": value })).unwrap();

        assert_eq!(
            options.perimeter_options().unwrap().wall_generator(),
            expected
        );
    }
}

#[test]
fn rejects_invalid_wall_generator_values() {
    for value in [json!("classic-ish"), json!(true), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "wall_generator": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(message)) if message.contains("wall_generator")
        ));
    }
}
