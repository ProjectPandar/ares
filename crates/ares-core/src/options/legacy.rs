use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, de::Error};
use serde_json::Value;

use super::SliceOptions;

mod token;
mod sla;
mod thumbnails;
mod wiping_volumes;

pub(crate) use self::token::{
    erases_percentage_for_absolute_keys, handle_legacy, handle_legacy_derived,
    handle_legacy_dest_bool,
};

impl<'de> Deserialize<'de> for SliceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = BTreeMap::<String, Value>::deserialize(deserializer)?;
        let values = normalize_legacy_options(values);
        let values = normalize_legacy_composite_options(values).map_err(D::Error::custom)?;
        Ok(Self { values })
    }
}

fn normalize_legacy_options(values: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut normalized = BTreeMap::new();
    for (key, value) in values {
        normalize_legacy_option_into(&mut normalized, key, value);
    }
    normalized
}

fn normalize_legacy_composite_options(
    mut values: BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>, String> {
    thumbnails::normalize_legacy_thumbnails(&mut values)?;
    wiping_volumes::normalize_legacy_wiping_volumes(&mut values)?;
    sla::normalize_legacy_sla(&mut values)?;
    // Config.cpp load_from_json/set_deserialize retains the full nullable vector.
    if let Some(value) = values.get_mut("inner_wall_speed") {
        let speed = serde_json::from_value::<super::OrcaFloatsNullable>(value.clone())
            .map_err(|error| error.to_string())?;
        *value = serde_json::to_value(speed).map_err(|error| error.to_string())?;
    }
    Ok(values)
}

fn normalize_legacy_option_into(values: &mut BTreeMap<String, Value>, key: String, value: Value) {
    // `load_from_json`-level derived keys (`Config.cpp:937-945`): a string
    // source `key`/token can produce extra entries (e.g. `is_infill_first`)
    // before `handle_legacy` rewrites the pair itself.
    if let Some(token) = value.as_str()
        && let Some((derived_key, derived_token)) =
            crate::options::legacy::handle_legacy_derived(&key, token)
    {
        values
            .entry(derived_key.to_owned())
            .or_insert(Value::Bool(derived_token.eq_ignore_ascii_case("true")));
    }
    if let Some((key, value)) = normalize_legacy_option((key, value)) {
        values.insert(key, value);
    }
}

fn normalize_legacy_option((key, value): (String, Value)) -> Option<(String, Value)> {
    // `set_deserialize` only hands `handle_legacy` the value string for
    // string-typed inputs (`Config.cpp:927-929`); for array values it gets an
    // empty token (`Config.cpp:957-962`), and non-scalar JSON values never
    // reach it. So a string entry may be renamed AND rewritten; a bool/number/
    // array is at most renamed — its original `Value` is preserved verbatim.
    let string_input = value.is_string();
    let original_key = key.clone();
    let mut token = value.as_str().unwrap_or_default().to_owned();
    let original_token = token.clone();
    let mut key = key;
    crate::options::handle_legacy(&mut key, &mut token);
    if key.is_empty() {
        return None;
    }
    // Non-string inputs keep their `Value` verbatim (at most renamed); a
    // string input that `handle_legacy` neither renamed nor rewrote is also
    // returned unchanged. Only a *rewritten* string token or a *rename* of a
    // string is re-materialized by the destination key.
    if !string_input || (key == original_key && token == original_token) {
        return Some((key, value));
    }
    let rewritten = if crate::options::legacy::handle_legacy_dest_bool(&key) {
        Value::Bool(token == "1" || token.eq_ignore_ascii_case("true"))
    } else {
        Value::String(token)
    };
    Some((key, rewritten))
}
