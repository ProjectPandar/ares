use std::collections::BTreeMap;

use serde_json::Value;

use crate::{ThumbnailParseError, parse_thumbnail_definitions};

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
    parse_thumbnail_definitions(thumbnails, default_format)
        .map_err(|error| thumbnail_legacy_error(error).to_owned())?;
    Ok(())
}

fn thumbnail_legacy_error(_error: ThumbnailParseError) -> &'static str {
    "Invalid value provided for parameter thumbnails"
}
