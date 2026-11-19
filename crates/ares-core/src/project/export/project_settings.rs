use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Value, ser::PrettyFormatter};

use crate::options::{format_number, json_deserialize::decode};
use crate::project::raw_settings::RawConfigValue;
use crate::{LoadSettingsRequest, OptionValueKind, SliceError, option_definitions};

use super::{BAMBU_VERSION, deferred, invalid};

pub(super) fn write(request: &LoadSettingsRequest<'_>) -> Result<Vec<u8>, SliceError> {
    if request.settings.len() != 2 || request.filaments.len() != 1 {
        return Err(deferred(
            "expected one flattened machine, process, and filament preset",
        ));
    }
    let mut raw = crate::profiles::export_settings(request)?;
    if raw
        .get("printer_technology")
        .and_then(RawConfigValue::token)
        .as_deref()
        != Some("FFF")
        || raw
            .get("nozzle_diameter")
            .is_some_and(|nozzles| nozzles.len() > 1)
    {
        return Err(deferred("expected a single-nozzle FFF machine"));
    }
    // The load-settings CLI resizes the flush table to the logical
    // filament count squared per nozzle (OrcaSlicer.cpp:3944–3945).
    raw.insert_strings("flush_volumes_matrix", &["0".into()]);
    for key in ["inherits_group", "different_settings_to_system"] {
        if !raw.contains_key(key) {
            raw.insert_strings(key, &[String::new(), String::new(), String::new()]);
        }
    }
    // `CLI::run` overlays the dynamic config on `FullPrintConfig` and
    // applies it back (`OrcaSlicer.cpp:4252–4256`) before persisting JSON.
    let mut output = BTreeMap::new();
    for def in option_definitions() {
        let value = match raw.get(def.key) {
            Some(value) => applied(def.key, def.kind, value.clone())?,
            None if def.full_config_default && full_print_config_default(def.key) => {
                from_token(def.kind, def.default_wire_value)?
            }
            None => continue,
        };
        output.insert(def.key, value);
    }
    for (key, value) in [
        ("from", "project"),
        ("name", "project_settings"),
        ("version", BAMBU_VERSION),
    ] {
        output.insert(key, Value::String(value.to_owned()));
    }
    let mut bytes = Vec::new();
    let mut ser =
        serde_json::Serializer::with_formatter(&mut bytes, PrettyFormatter::with_indent(b"\t"));
    output
        .serialize(&mut ser)
        .map_err(|err| invalid(err.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn full_print_config_default(key: &str) -> bool {
    // Host access lives in the device configuration; the deretract option
    // is registered as dynamic only (PrintConfig.cpp:6480–6483).
    !key.starts_with("printhost_")
        && !matches!(
            key,
            "print_host" | "print_host_webui" | "deretract_speed_extruder_change"
        )
}

fn applied(key: &str, kind: OptionValueKind, value: RawConfigValue) -> Result<Value, SliceError> {
    match (key, &value) {
        ("support_ironing_pattern", RawConfigValue::Text(s)) if s == "zig-zag" => {
            Ok(Value::String("rectilinear".to_owned()))
        }
        // Config.cpp:966 routes an empty JSON array through the empty-string
        // deserializer, where coFloats supplies a single zero.
        (_, RawConfigValue::List(values))
            if values.is_empty()
                && matches!(
                    kind,
                    OptionValueKind::Floats | OptionValueKind::FloatsNullable
                ) =>
        {
            Ok(Value::Array(vec![Value::String("0".to_owned())]))
        }
        _ => wire(kind, value),
    }
}

fn from_token(kind: OptionValueKind, token: &str) -> Result<Value, SliceError> {
    let value = decode(kind, token).map_err(invalid)?;
    wire(kind, value)
}

fn wire(kind: OptionValueKind, value: RawConfigValue) -> Result<Value, SliceError> {
    if let RawConfigValue::List(values) = value {
        return values
            .into_iter()
            .map(|value| wire_element(kind, value))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array);
    }
    if decoded_vector(kind) {
        let token = value
            .token()
            .ok_or_else(|| invalid("invalid project config vector"))?;
        let RawConfigValue::List(values) = decode(kind, &token).map_err(invalid)? else {
            return Err(invalid("invalid project config vector kind"));
        };
        return wire(kind, RawConfigValue::List(values));
    }
    wire_element(kind, value)
}

fn wire_element(kind: OptionValueKind, value: RawConfigValue) -> Result<Value, SliceError> {
    let text = match value {
        RawConfigValue::Text(text) => text,
        RawConfigValue::Flag(value) => u8::from(value).to_string(),
        RawConfigValue::Null => "nil".to_owned(),
        other => other
            .token()
            .ok_or_else(|| invalid("invalid project config value"))?,
    };
    if matches!(
        kind,
        OptionValueKind::Float | OptionValueKind::Floats | OptionValueKind::FloatsNullable
    ) && let Ok(value) = text.parse::<f64>()
    {
        return Ok(Value::String(format_number(value)));
    }
    Ok(Value::String(match kind {
        OptionValueKind::Percent
        | OptionValueKind::Percents
        | OptionValueKind::PercentsNullable
            if text.parse::<f64>().is_ok() =>
        {
            format!("{text}%")
        }
        OptionValueKind::Point => text.replace('x', ","),
        _ => text,
    }))
}

fn decoded_vector(kind: OptionValueKind) -> bool {
    use OptionValueKind::*;
    !matches!(
        kind,
        Float | FloatOrPercent | Percent | Int | Bool | Enum | String | Point
    )
}
