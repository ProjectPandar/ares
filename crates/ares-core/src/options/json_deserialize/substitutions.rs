use std::collections::BTreeMap;

use super::{encode_array, is_string_array};
use crate::ConfigSubstitution;
use crate::options::registry::{OptionValueKind, option_definition, option_metadata};
use crate::project::raw_settings::RawConfigValue;

/// `Config.cpp:654-672` substitution under the 3MF load's substitution
/// rule (the CLI passes `Enable`, `OrcaSlicer.cpp:1380, 1737`): a
/// scalar-bool wire that fails the "0"/"1" parse (`Config.hpp:1828-1848`)
/// retries the `looks_like_true` verdict (`Config.hpp:96-99`: trim +
/// case-insensitive "enabled"/"on"), and an enum-vector wire carrying an
/// unknown entry replaces the whole vector with the option default
/// (`Config.cpp:668-671`). Everything else keeps its value — the typed
/// builders still diagnose other failures. Returns each substitution in key
/// order, as `ConfigSubstitutionContext` records them.
pub(crate) fn normalize_unknown_wire_substitutions(
    values: &mut BTreeMap<String, RawConfigValue>,
) -> Vec<ConfigSubstitution> {
    let mut substitutions = Vec::new();
    for (key, value) in values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        match definition.kind {
            OptionValueKind::Bool => {
                let RawConfigValue::Text(text) = value else {
                    continue;
                };
                if matches!(text.split(',').next().unwrap_or_default().trim(), "0" | "1") {
                    continue;
                }
                let new_value = if looks_like_true(text) { "1" } else { "0" };
                substitutions.push(ConfigSubstitution::new(key, text, new_value));
                *value = RawConfigValue::Text(new_value.to_owned());
            }
            OptionValueKind::Enum => {
                let RawConfigValue::Text(text) = value else {
                    continue;
                };
                // Scalar coEnum substitution (`Config.cpp:657-676`): a key
                // missing from the option's enum TYPE map
                // (`ConfigOptionEnum::from_string`, `Config.hpp:2032-2040`)
                // replaces the whole option with the definition default
                // under the EnableSilent 3MF load rule. Keys valid for the
                // type but outside the option's own `enum_values` whitelist
                // (e.g. `monotonic` on `sparse_infill_pattern`) pass through
                // for the `Slic3r::validate` loop to reject, like native.
                let type_keys = definition
                    .enum_type_keys
                    .or_else(|| option_metadata(key).map(|metadata| metadata.enum_values));
                if type_keys.is_none_or(|keys| keys.contains(&text.trim())) {
                    continue;
                }
                substitutions.push(ConfigSubstitution::new(key, text, definition.default_value));
                *value = RawConfigValue::Text(definition.default_value.to_owned());
            }
            OptionValueKind::Enums | OptionValueKind::EnumsNullable => {
                let wire = match value {
                    // An empty vector is a valid vector (no substitution).
                    RawConfigValue::Text(text) if !text.is_empty() => text.clone(),
                    RawConfigValue::List(items) if is_string_array(items) => {
                        match encode_array(definition.kind, items) {
                            Ok(wire) => wire,
                            Err(_) => continue,
                        }
                    }
                    _ => continue,
                };
                let Some(metadata) = option_metadata(key) else {
                    continue;
                };
                // `ConfigOptionVector` splits the wire on commas
                // (`Config.hpp:2156-2178`) and each entry goes through the
                // enum key map. `nil` is only legal on the nullable kinds —
                // on the non-nullable ones Native throws instead of
                // substituting, so the wire passes through untouched for
                // the typed builder to diagnose. `std::getline` emits no
                // trailing empty item for a final comma, so the last empty
                // segment is dropped before validation.
                let mut entries: Vec<&str> = wire.split(',').map(str::trim).collect();
                if entries.last() == Some(&"") {
                    entries.pop();
                }
                if entries.contains(&"nil") && definition.kind != OptionValueKind::EnumsNullable {
                    continue;
                }
                let type_keys = definition.enum_type_keys.unwrap_or(metadata.enum_values);
                let entries_valid = |entry: &str| type_keys.contains(&entry) || entry == "nil";
                if entries.iter().copied().all(entries_valid) {
                    continue;
                }
                substitutions.push(ConfigSubstitution::new(
                    key,
                    &wire,
                    definition.default_value,
                ));
                *value = RawConfigValue::Text(definition.default_value.to_owned());
            }
            _ => {}
        }
    }
    substitutions
}

/// `ConfigHelpers::enum_looks_like_true_value` (`Config.hpp:96-99`).
fn looks_like_true(value: &str) -> bool {
    let value = value.trim();
    value.eq_ignore_ascii_case("enabled") || value.eq_ignore_ascii_case("on")
}
