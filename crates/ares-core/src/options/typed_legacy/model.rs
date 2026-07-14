use crate::{
    SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
};

use super::{
    EXPLICIT_RULES, LegacyOutcome, LegacyTransformError, transform_lexical, transform_obsolete,
};

pub(crate) fn deserialize_object_model_field(
    key: String,
    value: String,
    object: &mut ObjectOptionOverrides,
    region: &mut RegionOptionOverrides,
) -> Result<Option<(String, String)>, SliceError> {
    deserialize_model_field(key, value, Owners::Object { object, region })
}

pub(crate) fn deserialize_part_model_field(
    key: String,
    value: String,
    region: &mut RegionOptionOverrides,
) -> Result<Option<(String, String)>, SliceError> {
    deserialize_model_field(key, value, Owners::Part { region })
}

enum Owners<'a> {
    Object {
        object: &'a mut ObjectOptionOverrides,
        region: &'a mut RegionOptionOverrides,
    },
    Part {
        region: &'a mut RegionOptionOverrides,
    },
}

fn deserialize_model_field(
    key: String,
    value: String,
    mut owners: Owners<'_>,
) -> Result<Option<(String, String)>, SliceError> {
    if let Some(rule) = EXPLICIT_RULES.iter().find(|rule| rule.source == key) {
        return match transform_lexical(rule, &value) {
            LegacyOutcome::Assign { target, value } => {
                assign_or_retain(&mut owners, target, value, Some(rule.source))
            }
            LegacyOutcome::Consume => Ok(None),
            LegacyOutcome::Deferred { source, .. } => Err(SliceError::InvalidInput(format!(
                "unsupported deferred Orca model option {source}"
            ))),
            LegacyOutcome::Error(LegacyTransformError::InvalidArrayValue { source }) => Err(
                SliceError::InvalidInput(format!("invalid legacy Orca model option {source}")),
            ),
        };
    }

    if transform_obsolete(&key).is_some() {
        return Ok(None);
    }

    assign_or_retain(&mut owners, &key, value, None)
}

fn assign_or_retain(
    owners: &mut Owners<'_>,
    target: &str,
    value: String,
    legacy_source: Option<&str>,
) -> Result<Option<(String, String)>, SliceError> {
    let assigned = match owners {
        Owners::Object { object, region } => object
            .deserialize_known_field(target, &value)
            .and_then(|assigned| {
                if assigned {
                    Ok(true)
                } else {
                    region.deserialize_known_field(target, &value)
                }
            }),
        Owners::Part { region } => region.deserialize_known_field(target, &value),
    }
    .map_err(|error| match legacy_source {
        Some(source) => SliceError::InvalidInput(format!(
            "invalid legacy Orca model option {source} for {target}: {error}"
        )),
        None => error,
    })?;

    Ok((!assigned).then(|| (target.to_owned(), value)))
}
