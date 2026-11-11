//! Out-of-range validation of numeric options over the loaded flat config —
//! the `Slic3r::validate` `is_value_valid` loop (`PrintConfig.cpp:10519-10559`)
//! plus the `bridge_line_width` bound (`10504-10517`) and the O2
//! firmware-retraction rule, all feeding the single `std::map`-ordered
//! `Param values in 3mf/config error: ` map shared with O2.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::SliceError;
use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};

use super::super::SliceOptions;
use crate::OptionValueKind;
use crate::options::format_number;
use crate::options::registry::{self, OptionMetadata};

/// `Config.cpp:321-335::ConfigOptionDef::is_value_valid` with
/// `max_precision = 4` (`eps = 1e-4`). `ConfigOptionDef::min/max` default
/// to `-FLT_MAX`/`FLT_MAX` (Config.hpp:2478-2479), so an unset metadata
/// bound is unbounded, not skipped.
fn is_value_valid(value: f64, metadata: &OptionMetadata) -> bool {
    let min = metadata.min.unwrap_or(-(f32::MAX) as f64);
    let max = metadata.max.unwrap_or(f32::MAX as f64);
    if value.is_nan() {
        return metadata.nullable;
    }
    if min == 0.0 && value < 0.0 {
        return false;
    }
    const EP: f64 = 1e-4;
    if (value - min).abs() < EP || (value - max).abs() < EP {
        return true;
    }
    min <= value && value <= max
}

/// `Config.hpp:1013` coInts/coIntsNullable nil sentinel: the element is the
/// `INT_MAX` placeholder, which `is_value_valid` rejects against `max`.
const INT_NIL: f64 = i32::MAX as f64;

/// One config element normalized to the `is_value_valid` predicate input.
struct LoadedElement {
    /// Numeric predicate value; nil floats are `NaN`, nil ints `INT_NIL`.
    value: f64,
    /// Whether the source element was the `nil`/`null` placeholder.
    nil: bool,
    /// Whether the source element carried a `%` suffix (`FloatOrPercent` /
    /// `Percents` element form); re-appended by `serialize` (Config.hpp:1413).
    percent: bool,
}

/// `PrintConfig.cpp:10521-10559`: only numeric option types reach the loop.
fn is_numeric_kind(kind: OptionValueKind) -> bool {
    matches!(
        kind,
        OptionValueKind::Float
            | OptionValueKind::Percent
            | OptionValueKind::FloatOrPercent
            | OptionValueKind::Int
            | OptionValueKind::Floats
            | OptionValueKind::FloatsNullable
            | OptionValueKind::Percents
            | OptionValueKind::PercentsNullable
            | OptionValueKind::Ints
            | OptionValueKind::IntsNullable
    )
}

/// Whether the option's elements are integer-valued (nil → `INT_NIL`).
fn is_int_kind(kind: OptionValueKind) -> bool {
    matches!(
        kind,
        OptionValueKind::Int | OptionValueKind::Ints | OptionValueKind::IntsNullable
    )
}

/// Whether the option is a vector type — its scalar/serialized form is a
/// comma-joined list (`ConfigOptionVector::deserialize`, Config.hpp:868+).
fn is_vector_kind(kind: OptionValueKind) -> bool {
    matches!(
        kind,
        OptionValueKind::Floats
            | OptionValueKind::FloatsNullable
            | OptionValueKind::Percents
            | OptionValueKind::PercentsNullable
            | OptionValueKind::Ints
            | OptionValueKind::IntsNullable
    )
}

/// Parses one element text into the predicate value + nil flag.
/// `nil`/`null` elements carry the kind's nil sentinel; everything else
/// parses as a number (`%`-suffixed floats are the `FloatOrPercent`/
/// `Percents` element form, parsed numerically). `Err` means the element is
/// not a parseable numeric/nil token — upstream `set_deserialize` rejects it
/// before the range loop, surfaced here as `InvalidInput`.
fn parse_element(text: &str, int_kind: bool, key: &str) -> Result<LoadedElement, SliceError> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "nil" {
        return Ok(LoadedElement {
            value: if int_kind { INT_NIL } else { f64::NAN },
            nil: true,
            percent: false,
        });
    }
    let percent = trimmed.ends_with('%');
    let numeric = trimmed.strip_suffix('%').map_or(trimmed, str::trim);
    numeric
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|value| LoadedElement {
            value,
            nil: false,
            percent,
        })
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} contains invalid value")))
}

/// One element of a `RawConfigValue` — the post-`handle_legacy` token view
/// (`Text`/`Number`/`Flag` scalars, `List` vectors, `Null` nil).
fn raw_element(
    key: &str,
    value: &RawConfigValue,
    int_kind: bool,
) -> Result<LoadedElement, SliceError> {
    match value {
        RawConfigValue::Number(number) => number
            .as_f64()
            .filter(|value| value.is_finite())
            .map(|value| LoadedElement {
                value,
                nil: false,
                percent: false,
            })
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} contains invalid value"))),
        // A bool/list/object is not a numeric element — upstream
        // `deserialize` for a numeric option rejects it (`Config.cpp:1000`,
        // the `//should not happen` non-string branch).
        RawConfigValue::Flag(_) | RawConfigValue::List(_) | RawConfigValue::Object(_) => Err(
            SliceError::InvalidInput(format!("{key} contains invalid value")),
        ),
        RawConfigValue::Text(text) => parse_element(text, int_kind, key),
        RawConfigValue::Null => Ok(LoadedElement {
            value: if int_kind { INT_NIL } else { f64::NAN },
            nil: true,
            percent: false,
        }),
    }
}

/// Splits one config `RawConfigValue` into its elements. Vector-kind
/// scalars/text are comma-joined; `RawConfigValue::List` lists element-wise.
fn elements_of(
    key: &str,
    value: &RawConfigValue,
    int_kind: bool,
    vector_kind: bool,
) -> Result<Vec<LoadedElement>, SliceError> {
    match value {
        // A list on a scalar-kind option is a shape mismatch upstream
        // `deserialize` rejects before the range loop.
        RawConfigValue::List(_) if !vector_kind => Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        ))),
        RawConfigValue::List(values) => values
            .iter()
            .map(|item| raw_element(key, item, int_kind))
            .collect(),
        // `ConfigOptionVector::deserialize` splits a scalar token on ','.
        RawConfigValue::Text(text) if vector_kind => text
            .split(',')
            .map(|item| parse_element(item, int_kind, key))
            .collect(),
        _ => raw_element(key, value, int_kind).map(|element| vec![element]),
    }
}

/// `ConfigOption::serialize` / `ConfigOptionVector::serialize`: each element
/// is formatted from its value with the upstream `ss << v` `%g`-style 6
/// significant digits (`format_number`), comma-joined (Config.hpp:845-854).
/// nil elements render `nil` (Config.hpp:1070-1078); `FloatOrPercent`/`Percents`
/// elements re-append `%` (Config.hpp:1406-1417).
fn serialize_value(elements: &[LoadedElement]) -> String {
    elements
        .iter()
        .map(|element| {
            if element.nil {
                "nil".to_owned()
            } else {
                let mut token = format_number(element.value);
                if element.percent {
                    token.push('%');
                }
                token
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// `PrintConfig.cpp:10521-10559`: the generic `is_value_valid` loop over a
/// normalized flat `key → Value` config (the `cfg.keys()` iteration).
pub(in crate::options) fn collect_range_errors(
    values: &BTreeMap<String, RawConfigValue>,
    errors: &mut BTreeMap<String, String>,
) -> Result<(), SliceError> {
    for (key, value) in values {
        // `PrintConfig.cpp:8876-8878`: `Slic3r::validate` runs over
        // `FullPrintConfig` after `fpc.apply(*this, true)`, which drops the
        // dynamic `filament_extruder_override_keys` (`add_nullable`,
        // `:7288-7320`) — they are not `FullPrintConfig` static members, so
        // the `fpc.keys()` range loop never sees them. Keyed on the typed
        // `FilamentRetractOverrideOptions` field set, not the `filament_`
        // prefix: static `GCodeConfig` filament members stay checked.
        if crate::options::filament_options::is_dynamic_filament_override_key(key) {
            continue;
        }
        let Some(definition) = registry::option_definition(key) else {
            continue;
        };
        if !is_numeric_kind(definition.kind) {
            continue;
        }
        let Some(metadata) = registry::option_metadata(key) else {
            continue;
        };
        let int_kind = is_int_kind(definition.kind);
        let elements = elements_of(key, value, int_kind, is_vector_kind(definition.kind))?;
        // coInts nil elements already carry `INT_NIL` (> max → reject, even
        // when nullable); float nil elements carry `NaN` → nullable check.
        if elements
            .iter()
            .any(|element| !is_value_valid(element.value, metadata))
        {
            let min = metadata.min.unwrap_or(-(f32::MAX) as f64);
            let max = metadata.max.unwrap_or(f32::MAX as f64);
            let serialized = serialize_value(&elements);
            errors.entry(key.clone()).or_insert_with(|| {
                format!(
                    "{serialized} not in range [{}, {}]",
                    format_number(min),
                    format_number(max)
                )
            });
        }
    }
    Ok(())
}

impl SliceOptions {
    /// `PrintConfig.cpp:10519-10559`: the generic `is_value_valid` loop over
    /// the flat `--options` config. Replaces `LINE_WIDTH_RANGES`.
    /// `SliceOptions` deserialization already applies `handle_legacy`
    /// normalization, so `values()` holds the post-normalize config upstream
    /// `Slic3r::validate` sees; each entry is re-materialized as a
    /// `RawConfigValue` for the typed-element loop.
    pub fn validate_loaded_range_options(&self) -> Result<BTreeMap<String, String>, SliceError> {
        let mut raw = BTreeMap::new();
        for (key, value) in self.values() {
            raw.insert(
                key.clone(),
                RawConfigValue::deserialize(value.clone()).map_err(|error| {
                    SliceError::InvalidInput(format!("invalid option {key}: {error}"))
                })?,
            );
        }
        let mut errors = BTreeMap::new();
        collect_range_errors(&raw, &mut errors)?;
        Ok(errors)
    }

    /// `Slic3r::validate` over the loaded flat config: the shared O2+O3
    /// `std::map` serialized as the CLI's `key: text` lines.
    pub fn validate_loaded_config(&self) -> Result<(), SliceError> {
        loaded_config_error(self.validate_fff_options(false)?)
    }
}

impl ProjectSettingsRaw {
    /// `PrintConfig.cpp:10519-10559` over the merged project config. The raw
    /// view is already post-`handle_legacy` (`load.rs` normalizes it before
    /// this runs), so the range loop reads `RawConfigValue` elements
    /// directly — the same values upstream `Slic3r::validate` sees on the
    /// post-normalize config.
    pub(crate) fn loaded_range_options(
        &self,
        validate_print: bool,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();
        collect_range_errors(self.as_map(), &mut errors)?;
        // `PrintConfig.cpp:10504-10517`: the line-width nozzle bound (bridge
        // uses `min_nozzle_diameter`; the rest `5*max`) — same map, carried
        // via `SliceOptions`' typed readers rather than the raw element loop.
        let normalized =
            SliceOptions::deserialize(self.clone().into_object()).map_err(|error| {
                SliceError::InvalidInput(format!("invalid project settings: {error}"))
            })?;
        // `PrintConfig.cpp:3904-3920`: known flavor tokens must also be allowed.
        merge_first(&mut errors, normalized.validate_gcode_flavor_option()?);
        // `PrintConfig.cpp:10426-10434` belongs to validate(), not process().
        // Pure processing retains the existing Flow preflight error boundary.
        if validate_print {
            merge_first(&mut errors, normalized.validate_bridge_flow_options()?);
            // `PrintConfig.cpp:10436-10447`: strict-positive clearance values.
            merge_first(
                &mut errors,
                normalized.validate_extruder_clearance_options()?,
            );
            // `PrintConfig.cpp:10449-10454`: reject any nonpositive flow ratio.
            merge_first(
                &mut errors,
                normalized.validate_filament_flow_ratio_options()?,
            );
        }
        merge_first(&mut errors, normalized.validate_infill_pattern_options()?);
        merge_first(&mut errors, normalized.validate_extrusion_width_options()?);
        Ok(errors)
    }
}

impl crate::ProjectSettings {
    /// `Slic3r::validate` over the merged loaded config: O2 (typed fields)
    /// plus O3 (the raw flat map `raw`) into one key-sorted map, serialized
    /// as the CLI's `key: text` lines.
    pub(crate) fn validate_loaded_config(
        &self,
        raw: &ProjectSettingsRaw,
        validate_print: bool,
    ) -> Result<(), SliceError> {
        loaded_config_error(self.loaded_config_errors(raw, validate_print)?)
    }

    /// The `Slic3r::validate` error map: key to message.
    pub(crate) fn loaded_config_errors(
        &self,
        raw: &ProjectSettingsRaw,
        validate_print: bool,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = raw.loaded_range_options(validate_print)?;
        // O2 (firmware/wipe) writes the same std::map (PrintConfig.cpp:10391).
        merge_first(&mut errors, self.validate_loaded_retraction_map()?);
        Ok(errors)
    }
}

/// Serializes the shared error map into the `SliceError::InvalidValuesInProject`
/// multi-line text (`OrcaSlicer.cpp:3649-3653` prints `key: text` per line).
fn loaded_config_error(errors: BTreeMap<String, String>) -> Result<(), SliceError> {
    if errors.is_empty() {
        return Ok(());
    }
    let text = errors
        .iter()
        .map(|(key, message)| format!("{key}: {message}"))
        .collect::<Vec<_>>()
        .join("\n");
    Err(SliceError::InvalidValuesInProject(text))
}

fn merge_first(target: &mut BTreeMap<String, String>, source: BTreeMap<String, String>) {
    for (key, value) in source {
        target.entry(key).or_insert(value);
    }
}
