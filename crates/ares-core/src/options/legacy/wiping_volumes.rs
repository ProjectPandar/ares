use std::collections::BTreeMap;

use serde_json::Value;

use crate::options::parsing::parse_numeric_vector;

pub(super) fn normalize_legacy_wiping_volumes(
    values: &mut BTreeMap<String, Value>,
) -> Result<(), String> {
    if !values.contains_key("wiping_volumes_matrix")
        || values.contains_key("wiping_volumes_use_custom_matrix")
    {
        return Ok(());
    }

    let matrix = parse_numeric_vector("wiping_volumes_matrix", &values["wiping_volumes_matrix"])
        .map_err(|error| error.to_string())?;
    let extruder_count = (matrix.len() as f64).sqrt().round() as usize;
    let custom = matrix.iter().enumerate().any(|(index, value)| {
        let row = index / extruder_count;
        let column = index % extruder_count;
        row != column && !is_approximately_default_wiping_volume(*value)
    });
    values.insert(
        "wiping_volumes_use_custom_matrix".to_owned(),
        Value::Bool(custom),
    );
    Ok(())
}

fn is_approximately_default_wiping_volume(value: f64) -> bool {
    (value - 140.0).abs() < 1e-4
}
