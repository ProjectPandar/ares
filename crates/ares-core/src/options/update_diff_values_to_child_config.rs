use serde_json::Value;

use super::SliceOptions;
use crate::SliceError;

fn diff_child_variant_indices(
    current: &SliceOptions,
    target: &SliceOptions,
    extruder_id_name: &str,
    extruder_variant_name: &str,
) -> Result<Vec<isize>, SliceError> {
    let mut current_ids = Vec::new();
    let mut target_ids = Vec::new();
    if !extruder_id_name.is_empty() {
        current_ids =
            optional_int_vector(current.values().get(extruder_id_name), extruder_id_name)?;
        target_ids = optional_int_vector(target.values().get(extruder_id_name), extruder_id_name)?;
    }

    let current_variants = optional_string_vector(
        current.values().get(extruder_variant_name),
        extruder_variant_name,
    )?;
    let target_variants = optional_string_vector(
        target.values().get(extruder_variant_name),
        extruder_variant_name,
    )?;

    let current_variant_count = current_variants.len();
    let target_variant_count = target_variants.len();
    let mut variant_index = if current_variant_count > 0 {
        vec![-1; current_variant_count]
    } else {
        vec![0]
    };

    if target_variant_count == 0 {
        variant_index[0] = 0;
        return Ok(variant_index);
    }
    if !current_ids.is_empty() && current_variant_count != current_ids.len() {
        return Ok(variant_index);
    }
    if !target_ids.is_empty() && target_variant_count != target_ids.len() {
        return Ok(variant_index);
    }

    for (current_index, current_variant) in current_variants.iter().enumerate() {
        for (target_index, target_variant) in target_variants.iter().enumerate() {
            if current_variant == target_variant
                && (current_ids.is_empty()
                    || current_ids[current_index] == target_ids[target_index])
            {
                variant_index[current_index] = target_index as isize;
                break;
            }
        }
    }

    Ok(variant_index)
}

struct DiffDirectChildValueKeys<'a> {
    keys: &'a [&'a str],
    extruder_id_name: &'a str,
    extruder_variant_name: &'a str,
    key_set1: &'a [&'a str],
    key_set2: &'a [&'a str],
}

struct DiffChildConfigKeys<'a> {
    keys: &'a [&'a str],
    extruder_id_name: &'a str,
    extruder_variant_name: &'a str,
    key_set1: &'a [&'a str],
    key_set2: &'a [&'a str],
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M255 ports this helper before the full diff-to-child consumer"
    )
)]
fn apply_diff_direct_child_values(
    current: &mut SliceOptions,
    target: &SliceOptions,
    keys: DiffDirectChildValueKeys<'_>,
) {
    for key in keys.keys {
        if *key == keys.extruder_id_name || *key == keys.extruder_variant_name {
            continue;
        }
        let Some(current_value) = current.values().get(*key) else {
            continue;
        };
        let Some(target_value) = target.values().get(*key) else {
            continue;
        };
        if current_value == target_value {
            continue;
        }
        if !target_value.is_array()
            || (!keys.key_set1.contains(key)
                && (keys.key_set2.is_empty() || !keys.key_set2.contains(key)))
        {
            current
                .values
                .insert((*key).to_owned(), target_value.clone());
        }
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M257 ports this helper before public diff update wiring"
    )
)]
fn apply_diff_values_to_child_config(
    current: &mut SliceOptions,
    target: &SliceOptions,
    keys: DiffChildConfigKeys<'_>,
) -> Result<(), SliceError> {
    let variant_index = diff_child_variant_indices(
        current,
        target,
        keys.extruder_id_name,
        keys.extruder_variant_name,
    )?;

    for key in keys.keys {
        if *key == keys.extruder_id_name || *key == keys.extruder_variant_name {
            continue;
        }
        let Some(current_value) = current.values().get(*key) else {
            continue;
        };
        let Some(target_value) = target.values().get(*key) else {
            continue;
        };
        if current_value == target_value {
            continue;
        }
        if !target_value.is_array()
            || (!keys.key_set1.contains(key)
                && (keys.key_set2.is_empty() || !keys.key_set2.contains(key)))
        {
            current
                .values
                .insert((*key).to_owned(), target_value.clone());
            continue;
        }

        let mut source = json_vector(current_value, "diff vector source must be an array")?;
        let target = nullable_json_vector(target_value)?;
        let stride = diff_vector_stride(key, keys.key_set2);
        apply_diff_set_only_diff(&mut source, &target, &variant_index, stride)?;
        current
            .values
            .insert((*key).to_owned(), Value::Array(source));
    }

    Ok(())
}

fn diff_vector_stride(key: &str, key_set2: &[&str]) -> usize {
    if key_set2.contains(&key) { 2 } else { 1 }
}

fn apply_diff_set_only_diff<T: Clone>(
    source: &mut [T],
    target: &[Option<T>],
    diff_index: &[isize],
    stride: usize,
) -> Result<(), SliceError> {
    if source.len() != diff_index.len() * stride {
        return Err(SliceError::InvalidInput(
            "ConfigOptionVector::set_only_diff(): Assigning from an vector with invalid diff_index size".to_owned(),
        ));
    }

    for (source_index, target_index) in diff_index.iter().enumerate() {
        if *target_index == -1 {
            continue;
        }
        let target_offset = *target_index as usize * stride;
        if target[target_offset].is_none() {
            continue;
        }
        let source_offset = source_index * stride;
        for offset in 0..stride {
            source[source_offset + offset] = target[target_offset + offset]
                .as_ref()
                .expect("upstream checks nil only at the first stride slot")
                .clone();
        }
    }
    Ok(())
}

fn json_vector(value: &Value, message: &str) -> Result<Vec<Value>, SliceError> {
    value
        .as_array()
        .cloned()
        .ok_or_else(|| SliceError::InvalidInput(message.to_owned()))
}

fn nullable_json_vector(value: &Value) -> Result<Vec<Option<Value>>, SliceError> {
    Ok(value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput("diff vector target must be an array".to_owned()))?
        .iter()
        .map(|value| {
            if value.is_null() {
                None
            } else {
                Some(value.clone())
            }
        })
        .collect::<Vec<_>>())
}

fn optional_int_vector(value: Option<&Value>, key: &str) -> Result<Vec<i32>, SliceError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer vector")))?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain i32 integers")))
        })
        .collect()
}

fn optional_string_vector(value: Option<&Value>, key: &str) -> Result<Vec<String>, SliceError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string vector")))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain strings")))
        })
        .collect()
}

#[cfg(test)]
mod tests;
