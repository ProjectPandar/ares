use std::{fs, path::Path};

use ares_core::DownwardMachine;
use serde_json::Value;

use crate::{SliceArgs, failure::CliFailure};

pub(super) fn load(args: &SliceArgs) -> Result<Vec<DownwardMachine>, CliFailure> {
    if !args.downward_check {
        return Ok(Vec::new());
    }
    let paths = args
        .downward_settings
        .as_deref()
        .unwrap_or_default()
        .split(';')
        .filter(|path| !path.is_empty())
        .map(Path::new)
        .collect::<Vec<_>>();
    paths
        .into_iter()
        .map(|path| load_one(path, &args.output))
        .collect()
}

fn load_one(path: &Path, output: &Path) -> Result<DownwardMachine, CliFailure> {
    let bytes = fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CliFailure::downward_missing(path, output)
        } else {
            CliFailure::downward_config(output)
        }
    })?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|_| CliFailure::downward_config(output))?;
    if value.get("type").and_then(Value::as_str) != Some("machine")
        || value.get("from").and_then(Value::as_str) != Some("system")
    {
        return Err(CliFailure::downward_config(output));
    }
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| CliFailure::downward_config(output))?;
    let printable_area = area(value.get("printable_area"), output)?;
    let printable_height = value
        .get("printable_height")
        .and_then(number)
        .ok_or_else(|| CliFailure::downward_config(output))?;
    let exclude_area = area_optional(value.get("bed_exclude_area"), output)?;
    let wrapping_area = area_optional(value.get("wrapping_exclude_area"), output)?;
    Ok(DownwardMachine {
        name: name.to_owned(),
        printable_width: printable_area.0 as i32,
        printable_depth: printable_area.1 as i32,
        printable_height: printable_height as i32,
        exclude_width: exclude_area.map_or(0, |area| area.0 as i32),
        exclude_depth: exclude_area.map_or(0, |area| area.1 as i32),
        wrapping_width: wrapping_area.map_or(0, |area| area.0 as i32),
        wrapping_depth: wrapping_area.map_or(0, |area| area.1 as i32),
    })
}

fn area(value: Option<&Value>, output: &Path) -> Result<(f64, f64), CliFailure> {
    area_optional(value, output)?.ok_or_else(|| CliFailure::downward_config(output))
}

fn area_optional(value: Option<&Value>, output: &Path) -> Result<Option<(f64, f64)>, CliFailure> {
    let Some(values) = value else {
        return Ok(None);
    };
    let Some(values) = values.as_array() else {
        return Err(CliFailure::downward_config(output));
    };
    if values.is_empty() {
        return Ok(None);
    }
    if values.len() < 4 {
        return Err(CliFailure::downward_config(output));
    }
    let first = point(&values[0], output)?;
    let third = point(&values[2], output)?;
    Ok(Some((third.0 - first.0, third.1 - first.1)))
}

fn point(value: &Value, output: &Path) -> Result<(f64, f64), CliFailure> {
    let text = value
        .as_str()
        .ok_or_else(|| CliFailure::downward_config(output))?;
    let (x, y) = text
        .split_once('x')
        .ok_or_else(|| CliFailure::downward_config(output))?;
    let x = x
        .parse::<f64>()
        .map_err(|_| CliFailure::downward_config(output))?;
    let y = y
        .parse::<f64>()
        .map_err(|_| CliFailure::downward_config(output))?;
    Ok((x, y))
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
}
