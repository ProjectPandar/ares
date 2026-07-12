use std::collections::BTreeMap;

use serde_json::{Number, Value};

use crate::SliceError;

use super::SliceOptions;

impl SliceOptions {
    pub fn normalize_fdm(&mut self, used_filaments: usize) -> Result<(), SliceError> {
        if let Some(value) = self.values.get("extruder") {
            let extruder = parse_non_negative_integer("extruder", value)?;
            self.values.remove("extruder");
            if extruder != 0 {
                self.values
                    .entry("sparse_infill_filament".to_owned())
                    .or_insert_with(|| integer_value(extruder));
                self.values
                    .entry("wall_filament".to_owned())
                    .or_insert_with(|| integer_value(extruder));
            }
        }

        if !self.values.contains_key("solid_infill_filament")
            && let Some(value) = self.values.get("sparse_infill_filament")
        {
            let sparse_infill_filament =
                parse_non_negative_integer("sparse_infill_filament", value)?;
            self.values.insert(
                "solid_infill_filament".to_owned(),
                integer_value(sparse_infill_filament),
            );
        }

        if self
            .values
            .get("spiral_mode")
            .map(|value| parse_bool("spiral_mode", value))
            .transpose()?
            .unwrap_or(false)
        {
            disable_bool_array(&mut self.values, "retract_when_changing_layer", false)?;
            disable_bool_array(
                &mut self.values,
                "filament_retract_when_changing_layer",
                true,
            )?;
            self.values
                .insert("wall_loops".to_owned(), integer_value(1));
            self.values
                .insert("alternate_extra_wall".to_owned(), Value::Bool(false));
            self.values
                .insert("top_shell_layers".to_owned(), integer_value(0));
            self.values
                .insert("sparse_infill_density".to_owned(), integer_value(0));
        }

        if let Some(value) = self.values.get("resolution") {
            let resolution = parse_non_negative_float("resolution", value)?.max(0.001);
            self.values
                .insert("resolution".to_owned(), float_value(resolution));
        }

        if used_filaments > 0 && self.values.contains_key("enable_prime_tower") {
            self.values
                .entry("independent_support_layer_height".to_owned())
                .or_insert(Value::Bool(true));
            let mut enable_prime_tower =
                parse_bool("enable_prime_tower", &self.values["enable_prime_tower"])?;
            let print_sequence = parse_print_sequence(self.values.get("print_sequence"))?;
            let is_smooth_timelapse = parse_timelapse_type(self.values.get("timelapse_type"))?
                .is_some_and(|timelapse_type| timelapse_type == "1");

            if !is_smooth_timelapse && (used_filaments == 1 || print_sequence == "by object") {
                enable_prime_tower = false;
                self.values
                    .insert("enable_prime_tower".to_owned(), Value::Bool(false));
            }

            if enable_prime_tower {
                self.values.insert(
                    "independent_support_layer_height".to_owned(),
                    Value::Bool(false),
                );
            }
        }

        Ok(())
    }

    pub fn normalize_fdm_2(
        &mut self,
        num_objects: usize,
        used_filaments: usize,
    ) -> Result<Vec<String>, SliceError> {
        let mut changed_keys = Vec::new();
        if used_filaments == 0 || !self.values.contains_key("enable_prime_tower") {
            return Ok(changed_keys);
        }

        self.values
            .entry("independent_support_layer_height".to_owned())
            .or_insert(Value::Bool(true));
        let mut enable_prime_tower =
            parse_bool("enable_prime_tower", &self.values["enable_prime_tower"])?;
        let print_sequence = parse_print_sequence(self.values.get("print_sequence"))?;
        let is_smooth_timelapse = parse_timelapse_type(self.values.get("timelapse_type"))?
            .is_some_and(|timelapse_type| timelapse_type == "1");
        let enable_wrapping = self
            .values
            .get("enable_wrapping_detection")
            .map(|value| parse_bool("enable_wrapping_detection", value))
            .transpose()?
            .unwrap_or(false);

        if !is_smooth_timelapse
            && !enable_wrapping
            && (used_filaments == 1 || print_sequence == "by object" && num_objects > 1)
            && enable_prime_tower
        {
            enable_prime_tower = false;
            self.values
                .insert("enable_prime_tower".to_owned(), Value::Bool(false));
            changed_keys.push("enable_prime_tower".to_owned());
        }

        if enable_prime_tower {
            let independent_support_layer_height = parse_bool(
                "independent_support_layer_height",
                &self.values["independent_support_layer_height"],
            )?;
            if independent_support_layer_height {
                self.values.insert(
                    "independent_support_layer_height".to_owned(),
                    Value::Bool(false),
                );
                changed_keys.push("independent_support_layer_height".to_owned());
            }
        }

        Ok(changed_keys)
    }
}

fn parse_bool(key: &str, value: &Value) -> Result<bool, SliceError> {
    value
        .as_bool()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a boolean")))
}

fn disable_bool_array(
    values: &mut BTreeMap<String, Value>,
    key: &str,
    allow_null: bool,
) -> Result<(), SliceError> {
    let len = match values.get(key) {
        Some(Value::Array(items)) => {
            if items
                .iter()
                .all(|item| item.is_boolean() || allow_null && item.is_null())
            {
                items.len()
            } else {
                return Err(SliceError::InvalidInput(format!(
                    "{key} must be a boolean array"
                )));
            }
        }
        Some(_) => {
            return Err(SliceError::InvalidInput(format!(
                "{key} must be a boolean array"
            )));
        }
        None => 1,
    };

    values.insert(key.to_owned(), Value::Array(vec![Value::Bool(false); len]));
    Ok(())
}

fn parse_non_negative_integer(key: &str, value: &Value) -> Result<i64, SliceError> {
    let parsed = match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer")))?;

    if parsed >= 0 {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must be a non-negative integer"
        )))
    }
}

fn parse_non_negative_float(key: &str, value: &Value) -> Result<f64, SliceError> {
    let parsed = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .filter(|parsed: &f64| parsed.is_finite())
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a finite number")))?;

    if parsed >= 0.0 {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must be a non-negative number"
        )))
    }
}

fn parse_print_sequence(value: Option<&Value>) -> Result<&str, SliceError> {
    match value {
        Some(Value::String(value)) if value == "by layer" || value == "by object" => Ok(value),
        Some(_) => Err(SliceError::InvalidInput(
            "print_sequence must be a supported Orca print sequence".to_owned(),
        )),
        None => Ok("by layer"),
    }
}

fn parse_timelapse_type(value: Option<&Value>) -> Result<Option<&str>, SliceError> {
    match value {
        Some(Value::String(value)) if value == "0" || value == "1" => Ok(Some(value)),
        Some(_) => Err(SliceError::InvalidInput(
            "timelapse_type must be a supported Orca timelapse type".to_owned(),
        )),
        None => Ok(None),
    }
}

fn integer_value(value: i64) -> Value {
    Value::Number(Number::from(value))
}

fn float_value(value: f64) -> Value {
    Value::Number(Number::from_f64(value).expect("finite float"))
}
