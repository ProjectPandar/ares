use std::collections::BTreeMap;

use serde_json::Value;

use crate::{GCodeThumbnailDefinition, ThumbnailParseError, parse_thumbnail_definitions};

pub(super) fn normalize_legacy_thumbnails(
    values: &mut BTreeMap<String, Value>,
) -> Result<(), String> {
    let Some(thumbnails) = values.get("thumbnails").and_then(Value::as_str) else {
        return Ok(());
    };
    if thumbnails.is_empty() {
        return Ok(());
    }

    let default_format = values.get("thumbnails_format").and_then(Value::as_str);
    let definitions = parse_thumbnail_definitions(thumbnails, default_format)
        .map_err(|error| thumbnail_legacy_error(error).to_owned())?;
    values.insert(
        "thumbnails".to_owned(),
        Value::String(format_thumbnail_definitions(&definitions)),
    );
    Ok(())
}

fn format_thumbnail_definitions(definitions: &[GCodeThumbnailDefinition]) -> String {
    definitions
        .iter()
        .map(|definition| {
            format!(
                "{}x{}/{}",
                format_dimension(definition.width),
                format_dimension(definition.height),
                definition.format.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_dimension(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as u64)
    } else {
        value.to_string()
    }
}

fn thumbnail_legacy_error(_error: ThumbnailParseError) -> &'static str {
    "Invalid value provided for parameter thumbnails"
}
