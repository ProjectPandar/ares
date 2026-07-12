use serde_json::Value;

use crate::SliceError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtruderIndexIdMapLookup<'a> {
    pub extruder_or_filament_id: i32,
    pub id_name: &'a str,
    pub extruder_type: &'a str,
    pub nozzle_volume_type: &'a str,
    pub variant_name: &'a str,
    pub stride: usize,
}

use super::SliceOptions;

impl SliceOptions {
    pub fn get_index_for_extruder_no_id(
        &self,
        extruder_type: &str,
        nozzle_volume_type: &str,
        variant_name: &str,
        stride: usize,
    ) -> Result<isize, SliceError> {
        let Some(variants) = optional_string_vector(self.values().get(variant_name), variant_name)?
        else {
            return Ok(-1);
        };
        let target = extruder_variant_string(extruder_type, nozzle_volume_type)?;

        for index in 0..variants.len() {
            if string_get_at(&variants, variant_name, index)? == target {
                return checked_index(index, stride);
            }
        }

        Ok(-1)
    }

    pub fn get_index_for_extruder_complete_id_map(
        &self,
        lookup: ExtruderIndexIdMapLookup<'_>,
    ) -> Result<isize, SliceError> {
        let Some(variants) =
            optional_string_vector(self.values().get(lookup.variant_name), lookup.variant_name)?
        else {
            return Ok(-1);
        };
        let ids = complete_int_vector(
            self.values().get(lookup.id_name),
            lookup.id_name,
            variants.len(),
        )?;
        let target = extruder_variant_string(lookup.extruder_type, lookup.nozzle_volume_type)?;

        for index in 0..variants.len() {
            if string_get_at(&variants, lookup.variant_name, index)? == target
                && int_get_at(&ids, lookup.id_name, index)? == lookup.extruder_or_filament_id
            {
                return checked_index(index, lookup.stride);
            }
        }

        Ok(-1)
    }

    pub fn get_index_for_extruder_generated_id_map(
        &self,
        lookup: ExtruderIndexIdMapLookup<'_>,
    ) -> Result<isize, SliceError> {
        let Some(variants) =
            optional_string_vector(self.values().get(lookup.variant_name), lookup.variant_name)?
        else {
            return Ok(-1);
        };
        incomplete_int_vector(
            self.values().get(lookup.id_name),
            lookup.id_name,
            variants.len(),
        )?;
        let extruder_variant_list = optional_string_vector(
            self.values().get("extruder_variant_list"),
            "extruder_variant_list",
        )?;
        let target = extruder_variant_string(lookup.extruder_type, lookup.nozzle_volume_type)?;

        for index in 0..variants.len() {
            if string_get_at(&variants, lookup.variant_name, index)? == target
                && generated_extruder_id(extruder_variant_list.as_deref(), index)
                    == lookup.extruder_or_filament_id
            {
                return checked_index(index, lookup.stride);
            }
        }

        Ok(-1)
    }
}

fn extruder_variant_string(
    extruder_type: &str,
    nozzle_volume_type: &str,
) -> Result<String, SliceError> {
    let extruder_type = match extruder_type {
        "Direct Drive" => "Direct Drive",
        "Bowden" => "Bowden",
        _ => {
            return Err(SliceError::InvalidInput(
                "extruder_type has unknown enum value".to_owned(),
            ));
        }
    };
    let nozzle_volume_type = match nozzle_volume_type {
        "Standard" => "Standard",
        "High Flow" => "High Flow",
        _ => {
            return Err(SliceError::InvalidInput(
                "nozzle_volume_type has unknown enum value".to_owned(),
            ));
        }
    };
    Ok(format!("{extruder_type} {nozzle_volume_type}"))
}

fn optional_string_vector(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<Vec<String>>, SliceError> {
    let Some(value) = value else { return Ok(None) };
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
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn string_get_at<'a>(values: &'a [String], key: &str, id: usize) -> Result<&'a str, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .map(String::as_str)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}

fn checked_index(index: usize, stride: usize) -> Result<isize, SliceError> {
    index
        .checked_mul(stride)
        .and_then(|value| isize::try_from(value).ok())
        .ok_or_else(|| SliceError::InvalidInput("extruder index overflows isize".to_owned()))
}

fn complete_int_vector(
    value: Option<&Value>,
    key: &str,
    min_len: usize,
) -> Result<Vec<i32>, SliceError> {
    let values = int_vector(value, key)?;
    if values.len() < min_len {
        return Err(SliceError::InvalidInput(format!(
            "{key} must cover every variant"
        )));
    }
    Ok(values)
}

fn int_get_at(values: &[i32], key: &str, id: usize) -> Result<i32, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .copied()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}

fn incomplete_int_vector(
    value: Option<&Value>,
    key: &str,
    variant_len: usize,
) -> Result<Vec<i32>, SliceError> {
    let values = int_vector(value, key)?;
    if values.len() >= variant_len {
        return Err(SliceError::InvalidInput(format!(
            "{key} must be shorter than variant list"
        )));
    }
    Ok(values)
}

fn int_vector(value: Option<&Value>, key: &str) -> Result<Vec<i32>, SliceError> {
    let value = value
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer vector")))?;
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

fn generated_extruder_id(variants: Option<&[String]>, target_index: usize) -> i32 {
    let Some(variants) = variants else { return 0 };

    let mut variant_index = 0;
    for (extruder_index, variants_for_extruder) in variants.iter().enumerate() {
        for variant in split_comma_token_compress(variants_for_extruder) {
            if variant.trim().is_empty() {
                continue;
            }
            if variant_index == target_index {
                return extruder_index as i32 + 1;
            }
            variant_index += 1;
        }
    }
    0
}

fn split_comma_token_compress(variant: &str) -> Vec<&str> {
    if variant.is_empty() {
        return vec![""];
    }

    let mut tokens = Vec::new();
    let mut start = 0;
    let bytes = variant.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b',' {
            tokens.push(&variant[start..index]);
            while index + 1 < bytes.len() && bytes[index + 1] == b',' {
                index += 1;
            }
            start = index + 1;
        }
        index += 1;
    }
    tokens.push(&variant[start..]);
    tokens
}
