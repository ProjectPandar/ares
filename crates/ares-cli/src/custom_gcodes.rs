use std::{fs, io, path::Path};

use ares_core::{
    CustomGCodeInfo, CustomGCodeMode, SlicerContext,
    gcode_processor::{CustomGCodeItem, CustomGCodeType},
};
use serde_json::Value;

use crate::failure::CliFailure;

pub(super) fn load<'a>(
    context: SlicerContext<'a>,
    path: Option<&Path>,
    output: &Path,
    plate_index: i32,
) -> Result<SlicerContext<'a>, CliFailure> {
    let Some(path) = path.filter(|path| !path.as_os_str().is_empty()) else {
        return Ok(context);
    };
    let malformed = |detail: &str| {
        CliFailure::invalid_preset_file(output).with_stderr(format!(
            "run:Loading custom-gcode file \"{}\" failed: {detail}\n",
            path.display()
        ))
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                path,
                #[cfg(target_os = "linux")]
                output,
            )
            .with_stderr(format!(
                "run: can not find custom_gcode file: {}\n",
                path.display()
            )));
        }
        Err(error) => return Err(malformed(&error.to_string())),
    };
    let json: Value = serde_json::from_slice(&bytes).map_err(|error| {
        // nlohmann's stream parser reports the initial EOF at column 1.
        if bytes.is_empty() {
            malformed("[json.exception.parse_error.101] parse error at line 1, column 1: syntax error while parsing value - unexpected end of input; expected '[', '{', or a literal")
        } else {
            malformed(&error.to_string())
        }
    })?;
    let info = info(&json).map_err(malformed)?;
    Ok(context.with_custom_gcodes(plate_index, info))
}

fn info(json: &Value) -> Result<CustomGCodeInfo, &'static str> {
    let mode = match optional_string(json, "mode")? {
        "SingleExtruder" => CustomGCodeMode::SingleExtruder,
        "MultiAsSingle" => CustomGCodeMode::MultiAsSingle,
        "MultiExtruder" => CustomGCodeMode::MultiExtruder,
        _ => CustomGCodeMode::Undef,
    };
    let gcodes = json["gcodes"]
        .as_array()
        .ok_or("expected an array for 'gcodes'")?
        .iter()
        .map(item)
        .collect::<Result<_, _>>()?;
    Ok(CustomGCodeInfo { mode, gcodes })
}

fn item(json: &Value) -> Result<CustomGCodeItem, &'static str> {
    let kind = match json["type"]
        .as_str()
        .ok_or("expected a string for 'type'")?
    {
        "ColorChange" => CustomGCodeType::ColorChange,
        "PausePrint" => CustomGCodeType::PausePrint,
        "ToolChange" => CustomGCodeType::ToolChange,
        "Template" => CustomGCodeType::Template,
        "Custom" => CustomGCodeType::Custom,
        _ => CustomGCodeType::Unknown,
    };
    let print_z = json["print_z"]
        .as_f64()
        .ok_or("expected a number for 'print_z'")?;
    let extruder = json["extruder"]
        .as_f64()
        .filter(|&value| (0.0..=f64::from(i32::MAX)).contains(&value))
        .ok_or("expected a nonnegative tool ID for 'extruder'")? as u32;
    let color = json["color"]
        .as_str()
        .ok_or("expected a string for 'color'")?
        .to_owned();
    let extra = optional_string(json, "extra")?.to_owned();
    Ok(CustomGCodeItem {
        print_z,
        kind,
        extruder,
        color,
        extra,
    })
}

fn optional_string<'a>(json: &'a Value, key: &str) -> Result<&'a str, &'static str> {
    json.get(key).map_or(Ok(""), |value| {
        value
            .as_str()
            .ok_or("expected a string for 'mode' or 'extra'")
    })
}
