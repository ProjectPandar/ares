//! JSON parser for `CustomGCode::Info` (`OrcaSlicer.cpp` `--load-custom-gcodes`).
use serde_json::Value;

use super::{CustomGCodeInfo, CustomGCodeMode};
use crate::gcode_processor::{CustomGCodeItem, CustomGCodeType};

const EMPTY_JSON_EOF: &str = "[json.exception.parse_error.101] parse error at line 1, column 1: syntax error while parsing value - unexpected end of input; expected '[', '{', or a literal";

impl CustomGCodeInfo {
    /// Parses a `--load-custom-gcodes` JSON payload into `CustomGCodeInfo`,
    /// preserving upstream's nlohmann EOF diagnostic on empty input.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, String> {
        let json: Value = serde_json::from_slice(bytes).map_err(|error| {
            // nlohmann's stream parser reports the initial EOF at column 1.
            if bytes.is_empty() {
                EMPTY_JSON_EOF.to_owned()
            } else {
                error.to_string()
            }
        })?;
        parse_info(&json).map_err(str::to_owned)
    }
}

fn parse_info(json: &Value) -> Result<CustomGCodeInfo, &'static str> {
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
        .map(parse_item)
        .collect::<Result<_, _>>()?;
    Ok(CustomGCodeInfo { mode, gcodes })
}

fn parse_item(json: &Value) -> Result<CustomGCodeItem, &'static str> {
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
