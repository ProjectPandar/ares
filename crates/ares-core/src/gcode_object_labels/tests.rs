use super::*;
use crate::{LayerPrintPaths, options::GCodeFlavor};

#[test]
fn disabled_state_never_emits_labels() {
    let mut state = ObjectLabelState::new(ObjectLabelConfig {
        labels_enabled: false,
        exclusion: None,
    });

    assert_eq!(state.before_first_object_move(), "");
    assert_eq!(state.after_last_object_move(), "");
}

#[test]
fn enabled_state_emits_one_start_and_one_stop() {
    let mut state = ObjectLabelState::new(ObjectLabelConfig {
        labels_enabled: true,
        exclusion: None,
    });

    assert_eq!(
        state.before_first_object_move(),
        "; printing object ares-object-0 id:0 copy 0\n"
    );
    assert_eq!(state.before_first_object_move(), "");
    assert_eq!(
        state.after_last_object_move(),
        "; stop printing object ares-object-0 id:0 copy 0\n"
    );
    assert_eq!(state.after_last_object_move(), "");
}

#[test]
fn enabled_state_does_not_stop_before_start() {
    let mut state = ObjectLabelState::new(ObjectLabelConfig {
        labels_enabled: true,
        exclusion: None,
    });

    assert_eq!(state.after_last_object_move(), "");
}

#[test]
fn enabled_state_emits_labels_before_exclusion_commands() {
    let config = ObjectLabelConfig::from_options(
        &options_with_json(serde_json::json!({
            "exclude_object": true,
        })),
        GCodeFlavor::Klipper,
    )
    .unwrap();
    let mut state = ObjectLabelState::new(config);

    assert_eq!(
        state.before_first_object_move(),
        "; printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_START NAME=ares-object-0\n"
    );
    assert_eq!(
        state.after_last_object_move(),
        "; stop printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_END NAME=ares-object-0\n"
    );
}

#[test]
fn exclude_object_defaults_to_disabled() {
    let config =
        ObjectLabelConfig::from_options(&SliceOptions::default(), GCodeFlavor::Klipper).unwrap();

    assert!(config.labels_enabled);
    assert_eq!(config.object_definition(&[]), "");
    let mut state = ObjectLabelState::new(config);
    assert_eq!(
        state.before_first_object_move(),
        "; printing object ares-object-0 id:0 copy 0\n"
    );
    assert_eq!(
        state.after_last_object_move(),
        "; stop printing object ares-object-0 id:0 copy 0\n"
    );
}

#[test]
fn exclude_object_reads_boolean_value_for_klipper() {
    let options = options_with_json(serde_json::json!({ "exclude_object": true }));
    let config = ObjectLabelConfig::from_options(&options, GCodeFlavor::Klipper).unwrap();

    let empty_layer = [LayerPrintPaths::new(0, 0.2, Vec::new())];
    assert_eq!(config.object_definition(&empty_layer), "");
    let mut state = ObjectLabelState::new(config);
    assert_eq!(
        state.before_first_object_move(),
        "; printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_START NAME=ares-object-0\n"
    );
    assert_eq!(
        state.after_last_object_move(),
        "; stop printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_END NAME=ares-object-0\n"
    );
}

#[test]
fn exclude_object_false_suppresses_exclusion_commands() {
    let options = options_with_json(serde_json::json!({ "exclude_object": false }));
    let config = ObjectLabelConfig::from_options(&options, GCodeFlavor::MarlinFirmware).unwrap();

    assert_eq!(config.object_definition(&[]), "");
    let mut state = ObjectLabelState::new(config);
    assert_eq!(
        state.before_first_object_move(),
        "; printing object ares-object-0 id:0 copy 0\n"
    );
    assert_eq!(
        state.after_last_object_move(),
        "; stop printing object ares-object-0 id:0 copy 0\n"
    );
}

#[test]
fn exclude_object_rejects_non_boolean_values() {
    let options = options_with_json(serde_json::json!({ "exclude_object": "true" }));

    let err = ObjectLabelConfig::from_options(&options, GCodeFlavor::Klipper).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert_eq!(err.to_string(), "exclude_object must be a boolean");
}

#[test]
fn unsupported_flavor_accepts_exclude_object_without_commands() {
    let options = options_with_json(serde_json::json!({ "exclude_object": true }));
    let config = ObjectLabelConfig::from_options(&options, GCodeFlavor::Repetier).unwrap();

    assert_eq!(config.object_definition(&[]), "");
    let mut state = ObjectLabelState::new(config);
    assert_eq!(
        state.before_first_object_move(),
        "; printing object ares-object-0 id:0 copy 0\n"
    );
    assert_eq!(
        state.after_last_object_move(),
        "; stop printing object ares-object-0 id:0 copy 0\n"
    );
}

#[test]
fn option_defaults_to_enabled() {
    assert!(enabled(&SliceOptions::default()).unwrap());
}

#[test]
fn option_reads_boolean_value() {
    let options: SliceOptions =
        serde_json::from_value(serde_json::json!({ "gcode_label_objects": false })).unwrap();

    assert!(!enabled(&options).unwrap());
}

#[test]
fn option_rejects_non_boolean_values() {
    let options: SliceOptions =
        serde_json::from_value(serde_json::json!({ "gcode_label_objects": "true" })).unwrap();

    let err = enabled(&options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert_eq!(err.to_string(), "gcode_label_objects must be a boolean");
}

fn options_with_json(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}
