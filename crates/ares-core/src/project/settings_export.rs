use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, de::value::StrDeserializer};
use serde_json::{Serializer, ser::PrettyFormatter};

use super::{
    load::load_project_with_preparation,
    raw_settings::{ProjectSettingsRaw, RawConfigValue},
};
use crate::{
    ConfigOverrides, ORCA_SLICER_COMPATIBILITY_VERSION, OptionValueKind as Kind, SliceError,
    option_definitions,
    options::{escape_scalar_string, flush_matrix, format_number, json_deserialize},
    profiles::is_connection_option,
};

/// Produce the standalone Orca `--export-settings` document for a saved FFF 3MF.
///
/// The project bytes are loaded and composed without selecting a plate or slicing.
pub fn export_project_settings(input: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let overrides = ConfigOverrides::default();
    let (mut project, preparation) = load_project_with_preparation(input, &overrides)?;
    let filament_count = project.cli_filament_count();
    project.initialize_cli_loaded_project_config(filament_count);
    let mut raw = project.documents().project_settings_raw.clone();
    // OrcaSlicer.cpp:3849–4023 prepares the missing matrix before default completion.
    if !raw.contains_key("flush_volumes_matrix") {
        let source = project.settings();
        let mut settings = source.clone();
        // The typed default contains an empty name, not a loaded filament slot.
        settings
            .project
            .preset
            .filament_settings_id
            .0
            .truncate(filament_count);
        flush_matrix::prepare(&mut settings, &raw, &preparation, &overrides)?;
        let colours = &settings.filament.gcode.filament_colour;
        if *colours != source.filament.gcode.filament_colour {
            raw.insert_strings("filament_colour", &colours.0);
        }
        let print = &settings.project.print;
        let original = &source.project.print;
        if print.flush_volumes_matrix != original.flush_volumes_matrix {
            raw.insert_strings(
                "flush_volumes_matrix",
                &print
                    .flush_volumes_matrix
                    .0
                    .iter()
                    .copied()
                    .map(format_number)
                    .collect::<Vec<_>>(),
            );
        }
        if print.flush_multiplier != original.flush_multiplier {
            raw.insert_strings(
                "flush_multiplier",
                &print
                    .flush_multiplier
                    .0
                    .iter()
                    .map(|value| format_number(value.0))
                    .collect::<Vec<_>>(),
            );
        }
    }
    export_loaded_settings(raw, filament_count)
}

pub(crate) fn export_loaded_settings(
    mut raw: ProjectSettingsRaw,
    filament_count: usize,
) -> Result<Vec<u8>, SliceError> {
    // OrcaSlicer.cpp:4250–4259: apply FullPrintConfig before the final layout of
    // per-variant vectors. The retained preset values already loaded take precedence.
    for definition in option_definitions() {
        if definition.full_config_default && !raw.contains_key(definition.key) {
            let value = json_deserialize::decode(definition.kind, definition.default_wire_value)
                .map_err(|error| {
                    SliceError::InvalidInput(format!("{}: {error}", definition.key))
                })?;
            raw.insert(definition.key.to_owned(), value);
        }
    }
    normalize_fdm(&mut raw);
    raw.normalize_filament_values_to_variants();
    for key in ["different_settings_to_system", "inherits_group"] {
        if !raw.contains_key(key) {
            raw.insert_strings(key, &vec![String::new(); filament_count + 2]);
        }
    }
    let mut document = BTreeMap::from([
        ("from", Setting::Scalar("project".to_owned())),
        ("name", Setting::Scalar("project_settings".to_owned())),
        (
            "version",
            Setting::Scalar(ORCA_SLICER_COMPATIBILITY_VERSION.to_owned()),
        ),
    ]);
    for definition in option_definitions() {
        if let Some(value) = raw.get(definition.key)
            && !is_connection_option(definition.key)
        {
            let value = setting(definition.key, definition.kind, value).map_err(|error| {
                SliceError::InvalidInput(format!("{}: {error}", definition.key))
            })?;
            document.insert(definition.key, value);
        }
    }
    let mut output = Vec::new();
    document
        .serialize(&mut Serializer::with_formatter(
            &mut output,
            PrettyFormatter::with_indent(b"\t"),
        ))
        .map_err(|error| SliceError::InvalidInput(error.to_string()))?;
    output.push(b'\n');
    Ok(output)
}

// DynamicPrintConfig::normalize_fdm (PrintConfig.cpp:10348–10422),
// called before action dispatch with no slice-time used-filament count.
fn normalize_fdm(raw: &mut super::raw_settings::ProjectSettingsRaw) {
    let resolution = raw
        .get("resolution")
        .and_then(RawConfigValue::token)
        .and_then(|token| {
            json_deserialize::scalar_token(Kind::Float, &token)
                .ok()?
                .parse::<f64>()
                .ok()
        });
    if resolution.is_some_and(|value| value < 0.001) {
        raw.insert(
            "resolution".to_owned(),
            RawConfigValue::Text("0.001".to_owned()),
        );
    }
    if !raw
        .get("spiral_mode")
        .and_then(RawConfigValue::token)
        .is_some_and(|value| value == "1")
    {
        return;
    }
    for key in [
        "retract_when_changing_layer",
        "filament_retract_when_changing_layer",
    ] {
        let count = raw.get(key).map_or(0, RawConfigValue::len);
        raw.insert(
            key.to_owned(),
            RawConfigValue::List(vec![RawConfigValue::Flag(false); count]),
        );
    }
    for (key, token) in [
        ("wall_loops", "1"),
        ("alternate_extra_wall", "0"),
        ("top_shell_layers", "0"),
        ("sparse_infill_density", "0%"),
    ] {
        raw.insert(key.to_owned(), RawConfigValue::Text(token.to_owned()));
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum Setting {
    Scalar(String),
    Vector(Vec<String>),
}

// ConfigBase::save_to_json (Config.cpp:1564–1589) uses serialize/vserialize,
// not the delimiter-joined wire for a vector or the escaped wire for ordinary strings.
fn setting(key: &str, kind: Kind, value: &RawConfigValue) -> Result<Setting, String> {
    if let RawConfigValue::List(values) = value {
        let items = values
            .iter()
            .map(|item| element(kind, item))
            .collect::<Result<_, _>>()?;
        return Ok(Setting::Vector(items));
    }
    let mut text = element(kind, value)?;
    if matches!(key, "bed_custom_texture" | "bed_custom_model") {
        text = escape_scalar_string(&text);
    }
    Ok(Setting::Scalar(text))
}

fn element(kind: Kind, value: &RawConfigValue) -> Result<String, String> {
    let text = match value {
        RawConfigValue::Text(text) => text.clone(),
        RawConfigValue::List(items) if kind == Kind::PointsGroups => {
            return items
                .iter()
                .map(|item| element(Kind::Points, item))
                .collect::<Result<Vec<_>, _>>()
                .map(|points| points.join(","));
        }
        RawConfigValue::Null => "nil".to_owned(),
        other => other
            .token()
            .ok_or_else(|| "expected a config option".to_owned())?,
    };
    if text == "nil" || text.is_empty() {
        return Ok(text);
    }
    match kind {
        Kind::Float | Kind::Floats | Kind::FloatsNullable => number(Kind::Float, &text),
        Kind::FloatOrPercent | Kind::FloatsOrPercents => number(Kind::FloatOrPercent, &text),
        Kind::Percent | Kind::Percents | Kind::PercentsNullable => number(Kind::Percent, &text),
        Kind::Int | Kind::Ints | Kind::IntsNullable => {
            json_deserialize::scalar_token(Kind::Int, &text)?
                .parse::<i32>()
                .map(|value| value.to_string())
                .map_err(|error| error.to_string())
        }
        Kind::Bool | Kind::Bools | Kind::BoolsNullable => {
            json_deserialize::scalar_token(Kind::Bool, &text).map(|text| text.into_owned())
        }
        Kind::Point => {
            let point =
                crate::Point2d::deserialize(StrDeserializer::<serde::de::value::Error>::new(&text))
                    .map_err(|error| error.to_string())?;
            Ok(format!(
                "{},{}",
                format_number(point.x),
                format_number(point.y)
            ))
        }
        Kind::String
        | Kind::Strings
        | Kind::Points
        | Kind::PointsGroups
        | Kind::Enum
        | Kind::Enums
        | Kind::EnumsNullable => Ok(text),
    }
}

fn number(kind: Kind, value: &str) -> Result<String, String> {
    let token = json_deserialize::scalar_token(kind, value)?;
    let number = token
        .trim_end_matches('%')
        .parse::<f64>()
        .map_err(|error| error.to_string())?;
    if !number.is_finite() {
        return Err("expected a finite number".to_owned());
    }
    let mut text = format_number(number);
    if kind == Kind::Percent || token.ends_with('%') {
        text.push('%');
    }
    Ok(text)
}
