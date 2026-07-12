use serde_json::Value;

use crate::SliceError;

const ENABLE_FILAMENT_LONG_RETRACTION: i64 = 2;

fn prepared_filament_override_value(
    key: &str,
    filament_value: &Value,
    enable_long_retraction_when_cut: Option<&Value>,
) -> Result<Value, SliceError> {
    if key != "long_retractions_when_cut" && key != "retraction_distances_when_cut" {
        return Ok(filament_value.clone());
    }

    if enable_long_retraction_value(enable_long_retraction_when_cut)?
        == ENABLE_FILAMENT_LONG_RETRACTION
    {
        return Ok(filament_value.clone());
    }

    nil_array_like(key, filament_value)
}

fn enable_long_retraction_value(value: Option<&Value>) -> Result<i64, SliceError> {
    value.and_then(Value::as_i64).ok_or_else(|| {
        SliceError::InvalidInput("enable_long_retraction_when_cut must be an integer".to_owned())
    })
}

fn nil_array_like(key: &str, value: &Value) -> Result<Value, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an array")))?;
    Ok(Value::Array(vec![
        Value::String("nil".to_owned());
        values.len()
    ]))
}

fn apply_vector_override(
    machine_values: &mut Vec<Value>,
    override_values: &[Value],
    default_index: &[isize],
    nullable_override: bool,
) -> Result<bool, SliceError> {
    if !nullable_override {
        if machine_values.as_slice() == override_values {
            return Ok(false);
        }
        machine_values.clear();
        machine_values.extend_from_slice(override_values);
        return Ok(true);
    }

    if machine_values.is_empty() || override_values.is_empty() {
        return Ok(false);
    }

    let default_values = machine_values.clone();
    machine_values.resize(override_values.len(), default_values[0].clone());

    let mut modified = false;
    for (index, override_value) in override_values.iter().enumerate() {
        if !is_nil(override_value) {
            machine_values[index] = override_value.clone();
            modified = true;
            continue;
        }

        machine_values[index] = default_index
            .get(index)
            .and_then(|default_index| usize::try_from(*default_index - 1).ok())
            .and_then(|default_index| default_values.get(default_index))
            .unwrap_or(&default_values[0])
            .clone();
    }

    Ok(modified)
}

#[expect(
    clippy::too_many_arguments,
    reason = "M260 mirrors the upstream compute_filament_override_value boundary"
)]
fn compute_filament_override_value(
    key: &str,
    old_machine_value: &Value,
    new_machine_value: &Value,
    new_filament_value: &Value,
    enable_long_retraction_when_cut: Option<&Value>,
    default_index: &[isize],
    nullable_override: bool,
    diff_keys: &mut Vec<String>,
    filament_overrides: &mut serde_json::Map<String, Value>,
) -> Result<bool, SliceError> {
    let prepared_filament =
        prepared_filament_override_value(key, new_filament_value, enable_long_retraction_when_cut)?;
    let mut computed_values = new_machine_value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} machine value must be an array")))?
        .clone();
    let prepared_values = prepared_filament.as_array().ok_or_else(|| {
        SliceError::InvalidInput(format!("{key} filament value must be an array"))
    })?;

    apply_vector_override(
        &mut computed_values,
        prepared_values,
        default_index,
        nullable_override,
    )?;

    let computed_value = Value::Array(computed_values);
    if old_machine_value == &computed_value {
        return Ok(false);
    }

    diff_keys.push(key.to_owned());
    filament_overrides.insert(key.to_owned(), computed_value);
    Ok(true)
}

#[expect(
    clippy::too_many_arguments,
    reason = "M261 mirrors the upstream Print filament override loop inputs"
)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M261 stages the upstream Print filament override loop before full wiring"
    )
)]
fn collect_filament_override_updates(
    old_machine_values: &serde_json::Map<String, Value>,
    new_machine_values: &serde_json::Map<String, Value>,
    new_full_config_values: &serde_json::Map<String, Value>,
    enable_long_retraction_when_cut: Option<&Value>,
    default_index: &[isize],
    nullable_override: bool,
    diff_keys: &mut Vec<String>,
    filament_overrides: &mut serde_json::Map<String, Value>,
) -> Result<(), SliceError> {
    for key in crate::options::registry::extruder_retract_keys() {
        let filament_key = format!("filament_{key}");
        let Some(new_filament_value) = new_full_config_values.get(&filament_key) else {
            continue;
        };
        let old_machine_value = old_machine_values.get(*key).ok_or_else(|| {
            SliceError::InvalidInput(format!("{key} old machine value is missing"))
        })?;
        let new_machine_value = new_machine_values.get(*key).ok_or_else(|| {
            SliceError::InvalidInput(format!("{key} new machine value is missing"))
        })?;

        compute_filament_override_value(
            key,
            old_machine_value,
            new_machine_value,
            new_filament_value,
            enable_long_retraction_when_cut,
            default_index,
            nullable_override,
            diff_keys,
            filament_overrides,
        )?;
    }

    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "M262 mirrors the upstream PrintApply print_config_diffs inputs"
)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M262 stages PrintApply print_config_diffs before public wiring"
    )
)]
fn collect_print_config_diff_updates(
    current_config_values: &serde_json::Map<String, Value>,
    new_full_config_values: &serde_json::Map<String, Value>,
    current_keys: &[String],
    plate_index: usize,
    enable_long_retraction_when_cut: Option<&Value>,
    default_index: &[isize],
    nullable_override: bool,
    diff_keys: &mut Vec<String>,
    filament_overrides: &mut serde_json::Map<String, Value>,
) -> Result<(), SliceError> {
    for key in current_keys {
        let old_value = current_config_values
            .get(key)
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} current value is missing")))?;
        let Some(new_value) = new_full_config_values.get(key) else {
            continue;
        };
        let filament_key = format!("filament_{key}");
        if crate::options::registry::extruder_retract_keys().contains(&key.as_str())
            && new_full_config_values.contains_key(&filament_key)
        {
            let filament_value = new_full_config_values
                .get(&filament_key)
                .expect("filament key presence checked above");
            compute_filament_override_value(
                key,
                old_value,
                new_value,
                filament_value,
                enable_long_retraction_when_cut,
                default_index,
                nullable_override,
                diff_keys,
                filament_overrides,
            )?;
            continue;
        }
        if old_value != new_value {
            if key == "wipe_tower_x" || key == "wipe_tower_y" {
                maybe_push_wipe_tower_plate_diff(
                    key,
                    old_value,
                    new_value,
                    plate_index,
                    diff_keys,
                )?;
            } else {
                diff_keys.push(key.clone());
            }
        }
    }

    Ok(())
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M263 stages full_print_config_diffs before public wiring"
    )
)]
fn collect_full_print_config_diff_updates(
    current_full_config_values: &serde_json::Map<String, Value>,
    new_full_config_values: &serde_json::Map<String, Value>,
    new_full_keys: &[String],
    plate_index: usize,
    diff_keys: &mut Vec<String>,
) -> Result<(), SliceError> {
    for key in new_full_keys {
        let new_value = new_full_config_values
            .get(key)
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} new full value is missing")))?;
        let Some(old_value) = current_full_config_values.get(key) else {
            diff_keys.push(key.clone());
            continue;
        };
        if old_value == new_value {
            continue;
        }
        if key == "wipe_tower_x" || key == "wipe_tower_y" {
            maybe_push_wipe_tower_plate_diff(key, old_value, new_value, plate_index, diff_keys)?;
        } else {
            diff_keys.push(key.clone());
        }
    }

    Ok(())
}

fn maybe_push_wipe_tower_plate_diff(
    key: &str,
    old_value: &Value,
    new_value: &Value,
    plate_index: usize,
    diff_keys: &mut Vec<String>,
) -> Result<(), SliceError> {
    let old_values = old_value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} old value must be an array")))?;
    let new_values = new_value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} new value must be an array")))?;
    match (old_values.get(plate_index), new_values.get(plate_index)) {
        (Some(old_value), Some(new_value)) if old_value != new_value => {
            diff_keys.push(key.to_owned());
        }
        (Some(_), None) | (None, Some(_)) => {
            diff_keys.push(key.to_owned());
        }
        _ => {}
    }
    Ok(())
}

fn is_nil(value: &Value) -> bool {
    value.as_str() == Some("nil")
}

#[cfg(test)]
mod full_print_diff_tests;
#[cfg(test)]
mod key_loop_tests;
#[cfg(test)]
mod print_diff_tests;
#[cfg(test)]
mod tests;
