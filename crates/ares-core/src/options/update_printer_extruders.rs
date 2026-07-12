mod bools;
mod enums;
mod float_or_percent;
mod multiple_filament;
mod multiple_filament_bool;
mod multiple_filament_enum;
mod multiple_filament_float_or_percent;
mod numbers;
use super::{SliceOptions, registry::option_definition};
use crate::{ExtruderIndexIdMapLookup, OptionValueKind, SliceError};
use bools::copy_bool_values;
use enums::copy_enum_values;
use float_or_percent::copy_float_or_percent_values;
pub use multiple_filament::PrinterExtruderMultipleFilamentUpdate;
use numbers::copy_number_values;
use serde_json::{Number, Value};
use std::collections::BTreeSet;
pub struct PrinterExtruderUpdate<'a> {
    pub printer_config: &'a SliceOptions,
    pub key_set: &'a [&'a str],
    pub id_name: &'a str,
    pub variant_name: &'a str,
    pub stride: usize,
    pub extruder_id: usize,
}

impl SliceOptions {
    pub fn update_values_to_printer_extruders_string_int_keys(
        &mut self,
        update: PrinterExtruderUpdate<'_>,
    ) -> Result<(), SliceError> {
        let support = update.printer_config.support_different_extruders()?;
        if support.extruder_count <= 1 && !support.supported {
            return Ok(());
        }

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

        let variant_indices = self.printer_extruder_variant_indices(
            &extruder_types,
            &nozzle_volume_types,
            &update,
            support.extruder_count,
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
                OptionValueKind::Strings => Value::Array(copy_string_values(
                    value,
                    key,
                    &variant_indices,
                    update.stride,
                )?),
                OptionValueKind::Ints => Value::Array(copy_int_values(
                    value,
                    key,
                    &variant_indices,
                    update.stride,
                )?),
                OptionValueKind::Floats | OptionValueKind::Percent | OptionValueKind::Percents => {
                    Value::Array(copy_number_values(
                        value,
                        key,
                        &variant_indices,
                        update.stride,
                        false,
                    )?)
                }
                OptionValueKind::FloatsNullable | OptionValueKind::PercentsNullable => {
                    Value::Array(copy_number_values(
                        value,
                        key,
                        &variant_indices,
                        update.stride,
                        true,
                    )?)
                }
                OptionValueKind::FloatOrPercent => Value::Array(copy_float_or_percent_values(
                    value,
                    key,
                    &variant_indices,
                    update.stride,
                )?),
                OptionValueKind::Bools => Value::Array(copy_bool_values(
                    value,
                    key,
                    &variant_indices,
                    update.stride,
                    false,
                )?),
                OptionValueKind::BoolsNullable => Value::Array(copy_bool_values(
                    value,
                    key,
                    &variant_indices,
                    update.stride,
                    true,
                )?),
                OptionValueKind::Enums => Value::Array(copy_enum_values(
                    value,
                    key,
                    &variant_indices,
                    update.stride,
                    false,
                )?),
                OptionValueKind::EnumsNullable => Value::Array(copy_enum_values(
                    value,
                    key,
                    &variant_indices,
                    update.stride,
                    true,
                )?),
                _ => continue,
            };
            copied_values.push((key.to_owned(), value));
        }
        self.values.extend(copied_values);

        Ok(())
    }

    fn printer_extruder_variant_indices(
        &self,
        extruder_types: &[String],
        nozzle_volume_types: &[String],
        update: &PrinterExtruderUpdate<'_>,
        extruder_count: usize,
    ) -> Result<Vec<usize>, SliceError> {
        if (1..=extruder_count).contains(&update.extruder_id) {
            let index = update.extruder_id - 1;
            let variant_index =
                self.resolve_printer_extruder_variant_index(PrinterExtruderVariantLookup {
                    extruder_or_filament_id: update.extruder_id,
                    id_name: update.id_name,
                    extruder_type: string_get_at(extruder_types, "extruder_type", index)?,
                    nozzle_volume_type: string_get_at(
                        nozzle_volume_types,
                        "nozzle_volume_type",
                        index,
                    )?,
                    variant_name: update.variant_name,
                })?;
            if variant_index < 0 {
                return Err(SliceError::InvalidInput(
                    "could not resolve selected printer extruder variant".to_owned(),
                ));
            }
            return Ok(vec![variant_index as usize]);
        }

        (0..extruder_count)
            .map(|index| {
                let variant_index =
                    self.resolve_printer_extruder_variant_index(PrinterExtruderVariantLookup {
                        extruder_or_filament_id: index + 1,
                        id_name: update.id_name,
                        extruder_type: string_get_at(extruder_types, "extruder_type", index)?,
                        nozzle_volume_type: string_get_at(
                            nozzle_volume_types,
                            "nozzle_volume_type",
                            index,
                        )?,
                        variant_name: update.variant_name,
                    })?;
                Ok(if variant_index < 0 {
                    0
                } else {
                    variant_index as usize
                })
            })
            .collect()
    }

    fn resolve_printer_extruder_variant_index(
        &self,
        lookup: PrinterExtruderVariantLookup<'_>,
    ) -> Result<isize, SliceError> {
        if lookup.id_name.is_empty() || !self.values().contains_key(lookup.id_name) {
            return self.get_index_for_extruder_no_id(
                lookup.extruder_type,
                lookup.nozzle_volume_type,
                lookup.variant_name,
                1,
            );
        }

        let Some(variant_len) =
            string_vector_len(self.values().get(lookup.variant_name), lookup.variant_name)?
        else {
            return Ok(-1);
        };
        let id_map_lookup = ExtruderIndexIdMapLookup {
            extruder_or_filament_id: i32::try_from(lookup.extruder_or_filament_id)
                .map_err(|_| SliceError::InvalidInput("extruder id overflows i32".to_owned()))?,
            id_name: lookup.id_name,
            extruder_type: lookup.extruder_type,
            nozzle_volume_type: lookup.nozzle_volume_type,
            variant_name: lookup.variant_name,
            stride: 1,
        };
        if array_len(self.values().get(lookup.id_name)) >= Some(variant_len) {
            self.get_index_for_extruder_complete_id_map(id_map_lookup)
        } else {
            self.get_index_for_extruder_generated_id_map(id_map_lookup)
        }
    }
}

struct PrinterExtruderVariantLookup<'a> {
    extruder_or_filament_id: usize,
    id_name: &'a str,
    extruder_type: &'a str,
    nozzle_volume_type: &'a str,
    variant_name: &'a str,
}

fn copy_string_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
    stride: usize,
) -> Result<Vec<Value>, SliceError> {
    let values = string_vector(value, key)?;
    let mut copied = Vec::with_capacity(variant_indices.len() * stride);
    for variant_index in variant_indices {
        for offset in 0..stride {
            let source_index = source_index(*variant_index, stride, offset)?;
            copied.push(Value::String(
                string_get_at(&values, key, source_index)?.to_owned(),
            ));
        }
    }
    Ok(copied)
}

fn copy_int_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
    stride: usize,
) -> Result<Vec<Value>, SliceError> {
    let values = int_vector(value, key)?;
    let mut copied = Vec::with_capacity(variant_indices.len() * stride);
    for variant_index in variant_indices {
        for offset in 0..stride {
            let source_index = source_index(*variant_index, stride, offset)?;
            copied.push(Value::Number(Number::from(int_get_at(
                &values,
                key,
                source_index,
            )?)));
        }
    }
    Ok(copied)
}

fn source_index(variant_index: usize, stride: usize, offset: usize) -> Result<usize, SliceError> {
    variant_index
        .checked_mul(stride)
        .and_then(|index| index.checked_add(offset))
        .ok_or_else(|| SliceError::InvalidInput("printer extruder index overflow".to_owned()))
}

fn optional_string_vector(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<Vec<String>>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    string_vector(value, key).map(Some)
}

fn string_vector(value: &Value, key: &str) -> Result<Vec<String>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string vector")))?;
    if values.is_empty() {
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

fn int_vector(value: &Value, key: &str) -> Result<Vec<i32>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer vector")))?;
    if values.is_empty() {
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

fn int_get_at(values: &[i32], key: &str, id: usize) -> Result<i32, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .copied()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}

fn array_len(value: Option<&Value>) -> Option<usize> {
    value.and_then(Value::as_array).map(Vec::len)
}
