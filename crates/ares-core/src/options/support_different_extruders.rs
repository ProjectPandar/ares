use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::Value;

use crate::SliceError;

use super::SliceOptions;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DifferentExtrudersSupport {
    pub supported: bool,
    pub extruder_count: usize,
}

impl SliceOptions {
    pub fn support_different_extruders(&self) -> Result<DifferentExtrudersSupport, SliceError> {
        if !self.values().contains_key("nozzle_diameter") {
            return Ok(support(false, self.nozzle_diameters()?.len()));
        }

        let extruder_count = self.nozzle_diameters()?.len();
        let Some(variants) = optional_string_vector(
            self.values().get("extruder_variant_list"),
            "extruder_variant_list",
        )?
        else {
            return Ok(support(false, extruder_count));
        };

        let mut variant_set = BTreeSet::new();
        for index in 0..extruder_count {
            for token in
                split_variant_tokens(string_get_at(&variants, "extruder_variant_list", index)?)
            {
                variant_set.insert(token);
            }
        }

        Ok(support(variant_set.len() > 1, extruder_count))
    }
}

fn support(supported: bool, extruder_count: usize) -> DifferentExtrudersSupport {
    DifferentExtrudersSupport {
        supported,
        extruder_count,
    }
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

fn split_variant_tokens(variant: &str) -> Vec<String> {
    if variant.is_empty() {
        return vec![String::new()];
    }

    let mut tokens = Vec::new();
    let mut start = 0;
    let bytes = variant.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b',' {
            tokens.push(variant[start..index].to_owned());
            while index + 1 < bytes.len() && bytes[index + 1] == b',' {
                index += 1;
            }
            start = index + 1;
        }
        index += 1;
    }
    tokens.push(variant[start..].to_owned());
    tokens
}
