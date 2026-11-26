pub(crate) mod wire;

pub(crate) use wire::WireValue;

use serde::de::value::{Error as ValueError, SeqDeserializer, StrDeserializer, StringDeserializer};

use super::{
    ObjectOptionOverrides, RegionOptionOverrides,
    project_settings::ProjectSettingsBuilder,
    registry::{OptionValueKind, option_definition},
    typed_legacy::{
        LegacyOutcome, LegacyTransformError, find_explicit_rule, transform_lexical,
        transform_obsolete,
    },
};
use crate::SliceError;

pub(crate) fn deserialize_object_model_field(
    key: String,
    value: String,
    object: &mut ObjectOptionOverrides,
    region: &mut RegionOptionOverrides,
) -> Result<Option<(String, String)>, SliceError> {
    deserialize_model_field(key, value, Owners::Object { object, region })
}

pub(crate) fn deserialize_region_model_field(
    key: String,
    value: String,
    region: &mut RegionOptionOverrides,
) -> Result<Option<(String, String)>, SliceError> {
    deserialize_model_field(key, value, Owners::Region(region))
}

enum Owners<'a> {
    Object {
        object: &'a mut ObjectOptionOverrides,
        region: &'a mut RegionOptionOverrides,
    },
    Region(&'a mut RegionOptionOverrides),
}

fn deserialize_model_field(
    source: String,
    value: String,
    mut owners: Owners<'_>,
) -> Result<Option<(String, String)>, SliceError> {
    let Some((target, value)) = normalize_model_field(&source, value)? else {
        return Ok(None);
    };

    let assigned = match &mut owners {
        Owners::Object { object, region } => object
            .deserialize_known_field(target, &value)
            .and_then(|assigned| {
                if assigned {
                    Ok(true)
                } else {
                    region.deserialize_known_field(target, &value)
                }
            }),
        Owners::Region(region) => region.deserialize_known_field(target, &value),
    }
    .map_err(|error| legacy_error(&source, target, error))?;

    if assigned {
        return Ok(Some((target.to_owned(), value)));
    }

    validate_model_config_value(target, &value)
        .map(|_| None)
        .map_err(|error| legacy_error(&source, target, error))
}

fn normalize_model_field(
    source: &str,
    value: String,
) -> Result<Option<(&str, String)>, SliceError> {
    let Some(rule) = find_explicit_rule(source) else {
        return Ok(transform_obsolete(source)
            .is_none()
            .then_some((source, value)));
    };

    match transform_lexical(rule, &value) {
        LegacyOutcome::Assign { target, value } => Ok(Some((target, value))),
        LegacyOutcome::Consume => Ok(None),
        LegacyOutcome::Deferred { target, .. } => Ok(Some((target.unwrap_or(source), value))),
        LegacyOutcome::Error(LegacyTransformError::InvalidArrayValue { source }) => Err(
            SliceError::InvalidInput(format!("invalid legacy Orca model option {source}")),
        ),
    }
}

fn legacy_error(source: &str, target: &str, error: SliceError) -> SliceError {
    if source == target {
        SliceError::InvalidInput(bounded(&error.to_string(), 384))
    } else {
        SliceError::InvalidInput(format!(
            "invalid legacy Orca model option {} for {target}: {}",
            bounded(source, 96),
            bounded(&error.to_string(), 256),
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ModelConfigValueDestination {
    ProjectSettings,
    RegistryOnly,
}

pub(crate) fn validate_model_config_value(
    key: &str,
    value: &str,
) -> Result<ModelConfigValueDestination, SliceError> {
    let definition = option_definition(key).ok_or_else(|| {
        SliceError::InvalidInput(format!("unknown model config option {}", bounded(key, 96)))
    })?;
    let handled = match definition.kind {
        OptionValueKind::Point
        | OptionValueKind::Float
        | OptionValueKind::FloatOrPercent
        | OptionValueKind::Percent
        | OptionValueKind::Int
        | OptionValueKind::Bool
        | OptionValueKind::Enum => ProjectSettingsBuilder::validate_known_value(
            key,
            StrDeserializer::<ValueError>::new(value),
        )
        .map_err(|error| invalid_value(key, value, &error.to_string()))?,
        kind => {
            let decoded =
                wire::decode(kind, value).map_err(|reason| invalid_value(key, value, &reason))?;
            validate_project_value(key, decoded)
                .map_err(|error| invalid_value(key, value, &error.to_string()))?
        }
    };
    if handled {
        return Ok(ModelConfigValueDestination::ProjectSettings);
    }

    validate_registry_only(key, definition.kind, value)
        .map_err(|reason| invalid_value(key, value, &reason))?;
    Ok(ModelConfigValueDestination::RegistryOnly)
}

pub(crate) fn decode_wire(kind: OptionValueKind, value: &str) -> Result<WireValue, String> {
    wire::decode(kind, value)
}

fn validate_project_value(key: &str, value: WireValue) -> Result<bool, ValueError> {
    match value {
        WireValue::Scalar(value) => {
            ProjectSettingsBuilder::validate_known_value(key, StringDeserializer::new(value))
        }
        WireValue::Sequence(values) => ProjectSettingsBuilder::validate_known_value(
            key,
            SeqDeserializer::new(values.into_iter()),
        ),
    }
}

fn validate_registry_only(key: &str, kind: OptionValueKind, value: &str) -> Result<(), String> {
    match kind {
        OptionValueKind::Enum => validate_ownerless_enum(key, value),
        _ => wire::validate_lexical(kind, value),
    }
}

fn validate_ownerless_enum(key: &str, value: &str) -> Result<(), String> {
    let definition = option_definition(key).unwrap();
    let keys = definition
        .enum_type_keys
        .or_else(|| super::registry::option_metadata(key).map(|metadata| metadata.enum_values))
        .ok_or_else(|| "missing registry-only enum metadata".to_owned())?;
    if keys.contains(&value) {
        Ok(())
    } else {
        Err("unknown enum token".to_owned())
    }
}

fn invalid_value(key: &str, value: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!(
        "invalid model config option {key} value {:?}: {}",
        bounded(value, 96),
        bounded(reason, 256),
    ))
}

fn bounded(value: &str, limit: usize) -> String {
    let mut characters = value.chars();
    let bounded = characters.by_ref().take(limit).collect::<String>();
    if characters.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}
