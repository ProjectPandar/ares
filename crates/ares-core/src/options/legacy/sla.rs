use std::collections::BTreeMap;

use serde_json::{Number, Value};

pub(super) fn normalize_legacy_sla(values: &mut BTreeMap<String, Value>) -> Result<(), String> {
    normalize_correction(values, "relative_correction")?;
    normalize_correction(values, "material_correction")?;
    Ok(())
}

fn normalize_correction(values: &mut BTreeMap<String, Value>, key: &str) -> Result<(), String> {
    if !values.contains_key(key) {
        return Ok(());
    }

    let x_key = format!("{key}_x");
    let y_key = format!("{key}_y");
    let z_key = format!("{key}_z");
    let needs_x = !values.contains_key(&x_key);
    let needs_y = !values.contains_key(&y_key);
    let needs_z = !values.contains_key(&z_key);
    if !needs_x && !needs_y && !needs_z {
        return Ok(());
    }

    if needs_x || needs_y {
        let value = parse_numeric_at_index(key, &values[key], 0)?;
        if needs_x {
            insert_float(values, &x_key, value);
        }
        if needs_y {
            insert_float(values, &y_key, value);
        }
    }
    if needs_z {
        let value = parse_numeric_at_index(key, &values[key], 1)?;
        insert_float(values, &z_key, value);
    }
    Ok(())
}

fn parse_numeric_at_index(key: &str, value: &Value, index: usize) -> Result<f64, String> {
    let parsed = match value {
        Value::Number(number) if index == 0 => number.as_f64(),
        Value::String(text) => text
            .split([';', ','])
            .nth(index)
            .map(str::trim)
            .and_then(|part| {
                if part.is_empty() {
                    None
                } else {
                    part.parse().ok()
                }
            }),
        Value::Array(values) => values.get(index).and_then(|value| match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse().ok(),
            _ => None,
        }),
        _ => None,
    }
    .ok_or_else(|| format!("{key} must contain finite number at index {index}"))?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(format!("{key} must contain finite number at index {index}"))
    }
}

fn insert_float(values: &mut BTreeMap<String, Value>, key: &str, value: f64) {
    values.insert(
        key.to_owned(),
        Value::Number(Number::from_f64(value).expect("finite correction value")),
    );
}
