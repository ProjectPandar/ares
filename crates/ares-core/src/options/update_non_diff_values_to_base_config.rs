use serde_json::Value;

use super::SliceOptions;
use crate::SliceError;

mod restore_vectors;

#[cfg(test)]
use restore_vectors::{
    apply_non_diff_stride1_set_with_restore, apply_non_diff_stride2_set_with_restore,
    non_diff_stride1_restore_sizes, non_diff_stride2_restore_sizes,
    normalize_non_diff_stride2_restore_pair, normalized_non_diff_stride1_target_temp,
    resize_non_diff_stride1_source,
};

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M246 ports this helper before the full non-diff consumer"
    )
)]
fn validate_non_diff_stride2_float_vectors(
    key: &str,
    source: &Value,
    target: &Value,
) -> Result<(), SliceError> {
    if is_json_float_vector(source) && is_json_float_vector(target) {
        Ok(())
    } else {
        Err(SliceError::InvalidInput(format!(
            "update_non_diff_values_to_base_config: key '{key}' is expected to be ConfigOptionFloats for stride=2."
        )))
    }
}

fn is_json_float_vector(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|values| values.iter().all(|value| value.as_f64().is_some()))
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M245 ports this helper before the full non-diff consumer"
    )
)]
fn non_diff_restore_stride_and_expected_size(
    key: &str,
    key_set2: &[&str],
    restore_n: usize,
) -> (usize, usize) {
    let stride = if key_set2.contains(&key) { 2 } else { 1 };
    (stride, restore_n * stride)
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M244 ports this helper before the full non-diff consumer"
    )
)]
fn non_diff_restore_skips_when_child_has_more_variants(
    current_variant_count: usize,
    target_variant_count: usize,
) -> bool {
    current_variant_count > target_variant_count
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M243 ports this helper before the full non-diff consumer"
    )
)]
fn different_key_keeps_current_value(
    key: &str,
    target: &SliceOptions,
    key_set1: &[&str],
    key_set2: &[&str],
) -> bool {
    let Some(target_value) = target.values().get(key) else {
        return true;
    };
    !target_value.is_array()
        || (!key_set1.contains(&key) && (key_set2.is_empty() || !key_set2.contains(&key)))
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M242 ports this helper before the full non-diff consumer"
    )
)]
fn apply_non_diff_direct_inheritance(
    current: &mut SliceOptions,
    target: &SliceOptions,
    keys: &[&str],
    different_keys: &[&str],
) {
    for key in keys {
        let Some(target_value) = target.values().get(*key) else {
            continue;
        };
        let Some(current_value) = current.values().get(*key) else {
            continue;
        };
        if current_value == target_value || different_keys.contains(key) {
            continue;
        }
        current
            .values
            .insert((*key).to_owned(), target_value.clone());
    }
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "M241 ports this helper before its later consumer")
)]
fn non_diff_variant_indices(
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

    let mut variant_index = vec![-1; target_variants.len()];
    if current_variants.is_empty() {
        if let Some(first) = variant_index.first_mut() {
            *first = 0;
        }
        return Ok(variant_index);
    }
    if !current_ids.is_empty() && current_variants.len() != current_ids.len() {
        return Ok(variant_index);
    }
    if !target_ids.is_empty() && target_variants.len() != target_ids.len() {
        return Ok(variant_index);
    }

    for (target_index, target_variant) in target_variants.iter().enumerate() {
        for (current_index, current_variant) in current_variants.iter().enumerate() {
            if target_variant == current_variant
                && (target_ids.is_empty()
                    || current_ids
                        .get(current_index)
                        .is_some_and(|current_id| target_ids[target_index] == *current_id))
            {
                variant_index[target_index] = current_index as isize;
                break;
            }
        }
    }

    Ok(variant_index)
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

#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "M240 helper is staged until the full non-diff consumer is assembled"
    )
)]
pub(super) fn normalize_stride2_floats(values: &mut Vec<f64>, expected_size: usize) {
    if expected_size == 0 {
        values.clear();
        return;
    }
    if values.is_empty() {
        values.resize(expected_size, 0.0);
        return;
    }

    let first = values[0];
    let second = values.get(1).copied().unwrap_or(first);
    if values.len() < 2 {
        values.resize(2, first);
        values[1] = second;
    }
    if !values.len().is_multiple_of(2) {
        values.push(second);
    }

    if values.len() > expected_size {
        values.truncate(expected_size);
        return;
    }

    let have_variants = values.len() / 2;
    let want_variants = expected_size / 2;
    values.resize(expected_size, 0.0);
    for variant_index in have_variants..want_variants {
        values[variant_index * 2] = first;
        if variant_index * 2 + 1 < values.len() {
            values[variant_index * 2 + 1] = second;
        }
    }
}

#[cfg(test)]
mod tests;
