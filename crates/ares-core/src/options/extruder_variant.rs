use serde_json::{Number, Value};

use super::SliceOptions;
use crate::SliceError;

const DEFAULT_EXTRUDER_VARIANT: &str = "Direct Drive Standard";

impl SliceOptions {
    pub fn extend_extruder_variant(&mut self, num_extruders: usize) -> Result<(), SliceError> {
        let mut variants = self.extruder_variant_list(num_extruders)?;
        variants.resize(
            num_extruders,
            variants
                .first()
                .cloned()
                .unwrap_or_else(|| DEFAULT_EXTRUDER_VARIANT.to_owned()),
        );

        let mut printer_extruder_ids = Vec::new();
        let mut printer_extruder_variants = Vec::new();
        for (index, variant) in variants.iter().enumerate() {
            for token in split_variant_tokens(variant) {
                printer_extruder_ids.push(Value::Number(Number::from(index + 1)));
                printer_extruder_variants.push(Value::String(token));
            }
        }

        self.values.insert(
            "extruder_variant_list".to_owned(),
            Value::Array(variants.into_iter().map(Value::String).collect()),
        );
        self.values.insert(
            "printer_extruder_id".to_owned(),
            Value::Array(printer_extruder_ids),
        );
        self.values.insert(
            "printer_extruder_variant".to_owned(),
            Value::Array(printer_extruder_variants),
        );
        Ok(())
    }

    fn extruder_variant_list(&self, num_extruders: usize) -> Result<Vec<String>, SliceError> {
        let Some(value) = self.values.get("extruder_variant_list") else {
            return Ok(vec![DEFAULT_EXTRUDER_VARIANT.to_owned(); num_extruders]);
        };
        let Value::Array(values) = value else {
            return Err(SliceError::InvalidInput(
                "extruder_variant_list must be a string array".to_owned(),
            ));
        };
        if num_extruders > 0 && values.is_empty() {
            return Err(SliceError::InvalidInput(
                "extruder_variant_list must not be empty".to_owned(),
            ));
        }
        values
            .iter()
            .map(|value| {
                value.as_str().map(str::to_owned).ok_or_else(|| {
                    SliceError::InvalidInput(
                        "extruder_variant_list must be a string array".to_owned(),
                    )
                })
            })
            .collect()
    }
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
