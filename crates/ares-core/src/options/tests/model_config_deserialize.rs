mod legacy;
mod owners;
mod registry_kinds;
mod scalar_enums;
mod vector_enums;
mod wire;

use crate::{
    SliceError,
    options::model_config_deserialize::{
        ModelConfigValueDestination, validate_model_config_value,
    },
};

fn assert_project_value(key: &str, value: &str) {
    assert_eq!(
        validate_model_config_value(key, value).unwrap(),
        ModelConfigValueDestination::ProjectSettings,
        "{key}={value:?}",
    );
}

fn assert_rejected(key: &str, value: &str) -> String {
    let error = validate_model_config_value(key, value).unwrap_err();
    let SliceError::InvalidInput(message) = error else {
        panic!("unexpected error for {key}={value:?}: {error:?}");
    };
    assert!(message.contains(key), "{message}");
    assert!(message.len() <= 512, "unbounded error: {message}");
    message
}

fn changed_case(value: &str) -> String {
    let mut changed = false;
    value
        .chars()
        .map(|character| {
            if changed || !character.is_ascii_alphabetic() {
                character
            } else {
                changed = true;
                if character.is_ascii_lowercase() {
                    character.to_ascii_uppercase()
                } else {
                    character.to_ascii_lowercase()
                }
            }
        })
        .collect()
}
