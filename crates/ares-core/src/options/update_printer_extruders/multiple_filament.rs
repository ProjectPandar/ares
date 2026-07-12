use std::collections::BTreeSet;

use serde_json::{Number, Value};

use super::SliceOptions;
use super::multiple_filament_bool::copy_bool_values;
use super::multiple_filament_enum::copy_enum_values;
use super::multiple_filament_float_or_percent::copy_float_or_percent_values;
use crate::{ExtruderIndexIdMapLookup, OptionValueKind, SliceError, option_definition};

pub struct PrinterExtruderMultipleFilamentUpdate<'a> {
    pub printer_config: &'a SliceOptions,
    pub key_set: &'a [&'a str],
    pub id_name: &'a str,
    pub variant_name: &'a str,
}

impl SliceOptions {
    pub fn update_values_to_printer_extruders_for_multiple_filaments_string_int_keys(
        &mut self,
        update: PrinterExtruderMultipleFilamentUpdate<'_>,
    ) -> Result<(), SliceError> {
        let support = update.printer_config.support_different_extruders()?;
        if support.extruder_count <= 1 && !support.supported {
            return Ok(());
        }

        let Some(filament_maps) = optional_int_vector(
            update.printer_config.values().get("filament_map"),
            "filament_map",
        )?
        else {
            return Ok(());
        };
        let Some(extruder_types) = optional_string_vector(
            update.printer_config.values().get("extruder_type"),
            "extruder_type",
        )?
        else {
            return Ok(());
        };
        let Some(nozzle_volume_types) = optional_string_vector(
            update.printer_config.values().get("nozzle_volume_type"),
            "nozzle_volume_type",
        )?
        else {
            return Ok(());
        };

        let variant_indices = multiple_filament_variant_indices(
            self,
            &filament_maps,
            &extruder_types,
            &nozzle_volume_types,
            &update,
        )?;
        let keys = update.key_set.iter().copied().collect::<BTreeSet<_>>();
        let mut copied_values = Vec::new();
        for key in keys {
            let Some(definition) = option_definition(key) else {
                continue;
            };
            let Some(value) = self.values().get(key) else {
                continue;
            };
            let value = match definition.kind {
                OptionValueKind::Strings => {
                    Value::Array(copy_string_values(value, key, &variant_indices)?)
                }
                OptionValueKind::Ints => {
                    Value::Array(copy_int_values(value, key, &variant_indices)?)
                }
                OptionValueKind::Floats | OptionValueKind::Percent | OptionValueKind::Percents => {
                    Value::Array(copy_number_values(value, key, &variant_indices, false)?)
                }
                OptionValueKind::FloatsNullable | OptionValueKind::PercentsNullable => {
                    Value::Array(copy_number_values(value, key, &variant_indices, true)?)
                }
                OptionValueKind::FloatOrPercent => {
                    Value::Array(copy_float_or_percent_values(value, key, &variant_indices)?)
                }
                OptionValueKind::Bools => {
                    Value::Array(copy_bool_values(value, key, &variant_indices, false)?)
                }
                OptionValueKind::BoolsNullable => {
                    Value::Array(copy_bool_values(value, key, &variant_indices, true)?)
                }
                OptionValueKind::Enums => {
                    Value::Array(copy_enum_values(value, key, &variant_indices, false)?)
                }
                OptionValueKind::EnumsNullable => {
                    Value::Array(copy_enum_values(value, key, &variant_indices, true)?)
                }
                _ => continue,
            };
            copied_values.push((key.to_owned(), value));
        }
        self.values.extend(copied_values);

        Ok(())
    }
}

fn multiple_filament_variant_indices(
    options: &SliceOptions,
    filament_maps: &[i32],
    extruder_types: &[String],
    nozzle_volume_types: &[String],
    update: &PrinterExtruderMultipleFilamentUpdate<'_>,
) -> Result<Vec<usize>, SliceError> {
    filament_maps
        .iter()
        .enumerate()
        .map(|(filament_index, filament_map)| {
            let mapped_index = filament_map
                .checked_sub(1)
                .filter(|index| *index >= 0)
                .and_then(|index| usize::try_from(index).ok())
                .ok_or_else(|| {
                    SliceError::InvalidInput(
                        "filament_map must contain positive extruder ids".to_owned(),
                    )
                })?;
            let variant_index = resolve_variant_index(
                options,
                MultipleFilamentVariantLookup {
                    filament_id: filament_index + 1,
                    id_name: update.id_name,
                    extruder_type: string_get_at(extruder_types, "extruder_type", mapped_index)?,
                    nozzle_volume_type: string_get_at(
                        nozzle_volume_types,
                        "nozzle_volume_type",
                        mapped_index,
                    )?,
                    variant_name: update.variant_name,
                },
            )?;
            if variant_index >= 0 {
                return Ok(variant_index as usize);
            }

            Ok(options
                .values()
                .get(update.id_name)
                .map(|value| {
                    fallback_variant_index_for_filament_id(
                        value,
                        update.id_name,
                        filament_index + 1,
                    )
                })
                .transpose()?
                .flatten()
                .unwrap_or(0))
        })
        .collect()
}

struct MultipleFilamentVariantLookup<'a> {
    filament_id: usize,
    id_name: &'a str,
    extruder_type: &'a str,
    nozzle_volume_type: &'a str,
    variant_name: &'a str,
}

fn resolve_variant_index(
    options: &SliceOptions,
    lookup: MultipleFilamentVariantLookup<'_>,
) -> Result<isize, SliceError> {
    if lookup.id_name.is_empty() || !options.values().contains_key(lookup.id_name) {
        return options.get_index_for_extruder_no_id(
            lookup.extruder_type,
            lookup.nozzle_volume_type,
            lookup.variant_name,
            1,
        );
    }

    let Some(variant_len) = string_vector_len(
        options.values().get(lookup.variant_name),
        lookup.variant_name,
    )?
    else {
        return Ok(-1);
    };
    let id_lookup = ExtruderIndexIdMapLookup {
        extruder_or_filament_id: i32::try_from(lookup.filament_id)
            .map_err(|_| SliceError::InvalidInput("filament id overflows i32".to_owned()))?,
        id_name: lookup.id_name,
        extruder_type: lookup.extruder_type,
        nozzle_volume_type: lookup.nozzle_volume_type,
        variant_name: lookup.variant_name,
        stride: 1,
    };
    if array_len(options.values().get(lookup.id_name)) >= Some(variant_len) {
        options.get_index_for_extruder_complete_id_map(id_lookup)
    } else {
        options.get_index_for_extruder_generated_id_map(id_lookup)
    }
}

fn copy_string_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
) -> Result<Vec<Value>, SliceError> {
    let values = string_vector(value, key, true)?;
    let mut copied = vec![Value::String(String::new()); variant_indices.len()];
    for (filament_index, variant_index) in variant_indices.iter().enumerate() {
        if *variant_index < values.len() {
            copied[filament_index] = Value::String(values[*variant_index].clone());
        }
    }
    Ok(copied)
}

fn copy_int_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
) -> Result<Vec<Value>, SliceError> {
    let values = int_vector(value, key, true)?;
    let mut copied = vec![Value::Number(Number::from(0)); variant_indices.len()];
    for (filament_index, variant_index) in variant_indices.iter().enumerate() {
        if *variant_index < values.len() {
            copied[filament_index] = Value::Number(Number::from(values[*variant_index]));
        }
    }
    Ok(copied)
}

#[derive(Clone, Copy)]
enum NullableNumber {
    Nil,
    Value(f64),
}

fn copy_number_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
    allow_nil: bool,
) -> Result<Vec<Value>, SliceError> {
    let values = number_vector(value, key, allow_nil, true)?;
    let mut copied = vec![Value::Number(Number::from(0)); variant_indices.len()];
    for (filament_index, variant_index) in variant_indices.iter().enumerate() {
        if *variant_index < values.len() {
            copied[filament_index] = number_value(values[*variant_index], key)?;
        }
    }
    Ok(copied)
}

fn number_vector(
    value: &Value,
    key: &str,
    allow_nil: bool,
    allow_empty: bool,
) -> Result<Vec<NullableNumber>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number vector")))?;
    if values.is_empty() && !allow_empty {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

    values
        .iter()
        .map(|value| match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|value| value.is_finite())
                .map(NullableNumber::Value)
                .ok_or_else(|| {
                    SliceError::InvalidInput(format!("{key} must contain finite numbers"))
                }),
            Value::String(text) if allow_nil && text == "nil" => Ok(NullableNumber::Nil),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain finite numbers"
            ))),
        })
        .collect()
}

fn number_value(value: NullableNumber, key: &str) -> Result<Value, SliceError> {
    match value {
        NullableNumber::Nil => Ok(Value::String("nil".to_owned())),
        NullableNumber::Value(value) => Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain finite numbers"))),
    }
}

fn optional_string_vector(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<Vec<String>>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    string_vector(value, key, false).map(Some)
}

fn optional_int_vector(value: Option<&Value>, key: &str) -> Result<Option<Vec<i32>>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    int_vector(value, key, false).map(Some)
}

fn string_vector(value: &Value, key: &str, allow_empty: bool) -> Result<Vec<String>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string vector")))?;
    if values.is_empty() && !allow_empty {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain strings")))
        })
        .collect()
}

fn string_vector_len(value: Option<&Value>, key: &str) -> Result<Option<usize>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string vector")))?;
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }
    Ok(Some(values.len()))
}

fn string_get_at<'a>(values: &'a [String], key: &str, id: usize) -> Result<&'a str, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .map(String::as_str)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}

fn int_vector(value: &Value, key: &str, allow_empty: bool) -> Result<Vec<i32>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer vector")))?;
    if values.is_empty() && !allow_empty {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

    values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain i32 integers")))
        })
        .collect()
}

fn fallback_variant_index_for_filament_id(
    value: &Value,
    key: &str,
    filament_id: usize,
) -> Result<Option<usize>, SliceError> {
    let ids = int_vector(value, key, false)?;
    let filament_id = i32::try_from(filament_id)
        .map_err(|_| SliceError::InvalidInput("filament id overflows i32".to_owned()))?;
    Ok(ids.iter().position(|id| *id == filament_id))
}

fn array_len(value: Option<&Value>) -> Option<usize> {
    value.and_then(Value::as_array).map(Vec::len)
}
