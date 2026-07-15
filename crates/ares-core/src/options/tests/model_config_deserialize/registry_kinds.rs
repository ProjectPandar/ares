use crate::options::{
    model_config_deserialize::{ModelConfigValueDestination, validate_model_config_value},
    registry::{OptionDefinition, OptionValueKind, option_definitions},
};

use super::{assert_rejected, changed_case};

const OWNERLESS_ENUMS: &[(&str, &[&str], &str)] = &[
    ("display_orientation", &["landscape", "portrait"], "portrait"),
    ("first_layer_sequence_choice", &["Auto", "Customize"], "Auto"),
    ("material_print_speed", &["slow", "fast"], "fast"),
    ("other_layers_sequence_choice", &["Auto", "Customize"], "Auto"),
    ("support_pillar_connection_mode", &["zigzag", "cross", "dynamic"], "dynamic"),
];

#[test]
fn project_and_registry_destinations_form_the_exact_partition() {
    let mut project = 0;
    let mut registry = 0;
    let mut registry_kinds = Vec::new();

    for definition in option_definitions() {
        match validate_model_config_value(definition.key, &metadata_default(definition)).unwrap() {
            ModelConfigValueDestination::ProjectSettings => project += 1,
            ModelConfigValueDestination::RegistryOnly => {
                registry += 1;
                if !registry_kinds.contains(&definition.kind) {
                    registry_kinds.push(definition.kind);
                }
            }
        }
    }

    assert_eq!((project, registry), (650, 101));
    let expected_kinds = [
            OptionValueKind::Bool,
            OptionValueKind::Enum,
            OptionValueKind::Float,
            OptionValueKind::Floats,
            OptionValueKind::Int,
            OptionValueKind::Percent,
            OptionValueKind::String,
            OptionValueKind::Strings,
        ];
    assert_eq!(registry_kinds.len(), expected_kinds.len());
    assert!(expected_kinds.iter().all(|kind| registry_kinds.contains(kind)));
}

#[test]
fn ownerless_enum_ledgers_and_defaults_are_exact() {
    let enum_definitions = option_definitions()
        .iter()
        .filter(|definition| definition.kind == OptionValueKind::Enum)
        .filter(|definition| {
            validate_model_config_value(definition.key, &metadata_default(definition)).unwrap()
                == ModelConfigValueDestination::RegistryOnly
        })
        .collect::<Vec<_>>();
    assert_eq!(enum_definitions.len(), OWNERLESS_ENUMS.len());

    for &(key, tokens, expected_default) in OWNERLESS_ENUMS {
        let definition = enum_definitions
            .iter()
            .find(|definition| definition.key == key)
            .unwrap();
        assert_eq!(definition.default_value, expected_default);
        for &token in tokens {
            assert_eq!(
                validate_model_config_value(key, token).unwrap(),
                ModelConfigValueDestination::RegistryOnly,
            );
        }
        assert_rejected(key, "__invalid__");
        assert_rejected(key, &format!(" {} ", tokens[0]));
        assert_rejected(key, &changed_case(tokens[0]));
    }
}

#[test]
fn every_registry_only_kind_rejects_malformed_values_with_bounded_keyed_errors() {
    for definition in option_definitions() {
        if validate_model_config_value(definition.key, &metadata_default(definition)).unwrap()
            != ModelConfigValueDestination::RegistryOnly
        {
            continue;
        }
        let invalid = match definition.kind {
            OptionValueKind::Bool => "__invalid__",
            OptionValueKind::Enum => "__invalid__",
            OptionValueKind::Float => "__invalid__",
            OptionValueKind::Floats => "1,__invalid__",
            OptionValueKind::Int => "1.5",
            OptionValueKind::Percent => "__invalid__%",
            OptionValueKind::String => "trailing\\",
            OptionValueKind::Strings => "\"unterminated",
            _ => continue,
        };
        assert_rejected(definition.key, invalid);
    }
}

#[test]
fn unknown_and_unported_aliases_preserve_the_exact_source_name() {
    assert_rejected("unknown_model_option", "1");
    assert_rejected("perimeter_feed_rate", "1");
    assert_rejected("display_orientation", &"x".repeat(10_000));
}

#[test]
fn unknown_option_errors_bound_attacker_controlled_keys() {
    let key = format!("unknown_{}", "x".repeat(10_000));
    let error = validate_model_config_value(&key, "1").unwrap_err();
    let crate::SliceError::InvalidInput(message) = error else {
        panic!("unexpected error: {error:?}");
    };

    assert!(message.contains("unknown_"), "{message}");
    assert!(message.ends_with("..."), "{message}");
    assert!(message.len() <= 512, "unbounded error: {message}");
    assert!(!message.contains(&key));
}

fn metadata_default(definition: &OptionDefinition) -> String {
    match definition.kind {
        OptionValueKind::Bool | OptionValueKind::Bools | OptionValueKind::BoolsNullable => {
            definition
                .default_value
                .split(',')
                .map(|value| match value.trim() {
                    "true" => "1",
                    "false" => "0",
                    other => other,
                })
                .collect::<Vec<_>>()
                .join(",")
        }
        _ => definition.default_value.to_owned(),
    }
}
