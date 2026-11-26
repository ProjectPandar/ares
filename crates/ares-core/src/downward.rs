use serde_json::Value;

use crate::SliceError;

#[derive(Clone, Debug, PartialEq)]
pub struct DownwardMachine {
    pub name: String,
    pub printable_width: i32,
    pub printable_depth: i32,
    pub printable_height: i32,
    pub exclude_width: i32,
    pub exclude_depth: i32,
    pub wrapping_width: i32,
    pub wrapping_depth: i32,
}

impl DownwardMachine {
    /// Parses one system machine preset JSON for downward-compatibility checks
    /// (`OrcaSlicer.cpp` `--downward-settings` machine loader).
    pub fn from_machine_json(bytes: &[u8]) -> Result<Self, SliceError> {
        let invalid = || SliceError::InvalidInput("invalid downward machine JSON".to_owned());
        let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        if value.get("type").and_then(Value::as_str) != Some("machine")
            || value.get("from").and_then(Value::as_str) != Some("system")
        {
            return Err(invalid());
        }
        let name = value
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(invalid)?;
        let printable_area = area_required(value.get("printable_area"))?;
        let printable_height = value
            .get("printable_height")
            .and_then(number)
            .ok_or_else(invalid)?;
        let exclude_area = area_optional(value.get("bed_exclude_area"))?;
        let wrapping_area = area_optional(value.get("wrapping_exclude_area"))?;
        Ok(Self {
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
}

fn area_required(value: Option<&Value>) -> Result<(f64, f64), SliceError> {
    area_optional(value)?
        .ok_or_else(|| SliceError::InvalidInput("invalid downward machine JSON".to_owned()))
}

fn area_optional(value: Option<&Value>) -> Result<Option<(f64, f64)>, SliceError> {
    let invalid = || SliceError::InvalidInput("invalid downward machine JSON".to_owned());
    let Some(values) = value else {
        return Ok(None);
    };
    let Some(values) = values.as_array() else {
        return Err(invalid());
    };
    if values.is_empty() {
        return Ok(None);
    }
    if values.len() < 4 {
        return Err(invalid());
    }
    let first = point(&values[0])?;
    let third = point(&values[2])?;
    Ok(Some((third.0 - first.0, third.1 - first.1)))
}

fn point(value: &Value) -> Result<(f64, f64), SliceError> {
    let invalid = || SliceError::InvalidInput("invalid downward machine JSON".to_owned());
    let text = value.as_str().ok_or_else(invalid)?;
    let (x, y) = text.split_once('x').ok_or_else(invalid)?;
    let x = x.parse::<f64>().map_err(|_| invalid())?;
    let y = y.parse::<f64>().map_err(|_| invalid())?;
    Ok((x, y))
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
}

pub fn compatible_downward_machines(
    object_size: [f64; 3],
    machines: &[DownwardMachine],
    wrapping_enabled: bool,
) -> Vec<String> {
    machines
        .iter()
        .filter(|machine| fits(object_size, machine, wrapping_enabled))
        .map(|machine| machine.name.clone())
        .collect()
}

pub(crate) type ProjectCheck<'a> = (&'a [DownwardMachine], &'a mut Vec<String>);

pub(crate) fn check_project(
    project: &crate::Project,
    selection: Option<u32>,
    check: Option<ProjectCheck<'_>>,
) {
    let Some((machines, report)) = check else {
        return;
    };
    let sizes = project.imported_plate_sizes(selection);
    let wrapping_enabled = project.settings().process.gcode.enable_wrapping_detection.0;
    report.extend(
        machines
            .iter()
            .filter(|machine| {
                sizes
                    .iter()
                    .all(|&size| fits(size, machine, wrapping_enabled))
            })
            .map(|machine| machine.name.clone()),
    );
}

fn fits(size: [f64; 3], machine: &DownwardMachine, wrapping_enabled: bool) -> bool {
    if size[2] > f64::from(machine.printable_height)
        || size[1] > f64::from(machine.printable_depth)
        || size[0] > f64::from(machine.printable_width)
    {
        return false;
    }
    if machine.exclude_width > 0 {
        let real_width = machine.printable_width - machine.exclude_width;
        let real_depth = machine.printable_depth - machine.exclude_depth;
        if size[0] > f64::from(real_width) && size[1] > f64::from(real_depth) {
            return false;
        }
    }
    !wrapping_enabled
        || machine.wrapping_width <= 0
        || size[1] <= f64::from(machine.printable_depth - machine.wrapping_depth)
}
