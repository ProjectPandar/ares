//! ConfigOptionFloatOrPercent tag/magnitude checks at bbs_3mf export bytes.
#[path = "application/cli_command.rs"]
mod cli_command;
#[path = "application/cli_command_directory.rs"]
mod cli_command_directory;
#[path = "../../support/project_preset_catalog.rs"]
mod project_preset_catalog;
#[path = "application/project_presets.rs"]
mod project_presets;
use super::stages::{FailureKind, Stage, StageError};
pub(crate) use cli_command::{CliCommand, cli_command};
pub(crate) use project_presets::{
    native_arguments, project_supply, slice_direct_project_plate, slice_plater_plate,
    slice_project, slice_project_plate, slice_project_plate_processed,
    slice_project_plate_processed_validated, slice_project_plate_validated,
};
use serde_json::{Map, Value};
use std::io::{Cursor, Read};

#[derive(Debug, PartialEq)]
pub(crate) struct WidthValue {
    pub(crate) value: f64,
    pub(crate) percent: bool,
}

pub(crate) fn width(value: &Value) -> Result<WidthValue, String> {
    let text = value.as_str().ok_or("expected scalar string")?;
    let (number, percent) = text
        .strip_suffix('%')
        .map_or((text, false), |number| (number, true));
    let value = number.parse::<f64>().map_err(|error| error.to_string())?;
    if !value.is_finite() {
        return Err("nonfinite width".into());
    }
    Ok(WidthValue { value, percent })
}

pub(crate) fn settings(project: &[u8]) -> Result<Map<String, Value>, StageError> {
    let error = |message| {
        StageError::new(
            Stage::Application,
            FailureKind::ApplicationMismatch,
            message,
        )
    };
    let mut archive =
        zip::ZipArchive::new(Cursor::new(project)).map_err(|e| error(e.to_string()))?;
    let mut bytes = Vec::new();
    archive
        .by_name("Metadata/project_settings.config")
        .map_err(|e| error(e.to_string()))?
        .read_to_end(&mut bytes)
        .map_err(|e| error(e.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|e| error(e.to_string()))
}

/// Generic requested-vs-effective check for non-width cases: the
/// exported `project_settings.config` must carry the option with a value
/// equal to the requested one (scalar-string equality after
/// serialization — `apply_override` already canonicalizes types).
/// Source-owned silent-mode and subthreshold-resolution normalization is explicit.
pub(crate) fn verify_value(
    project: &[u8],
    key: &str,
    requested: &Value,
) -> Result<Value, StageError> {
    let mismatch =
        |detail| StageError::new(Stage::Application, FailureKind::ApplicationMismatch, detail);
    let effective = settings(project)?;
    // `nil` probes: the option is nullable, so the legal export either
    // omits the key or serializes the literal `nil`.
    if requested == &Value::String("nil".to_owned()) {
        return match effective.get(key) {
            None | Some(Value::Null) => Ok(Value::String("nil".to_owned())),
            Some(actual) if actual.as_str() == Some("nil") => Ok(Value::String("nil".to_owned())),
            Some(actual) => Err(mismatch(format!(
                "{key}: requested nil, effective {actual}"
            ))),
        };
    }
    let actual = effective
        .get(key)
        .ok_or_else(|| mismatch(format!("missing {key}")))?;
    // `PrintConfig.cpp:8272-8278` consumes this key; :4444 defaults it to false.
    if key == "silent_mode" {
        if actual.as_str() != Some("0") {
            return Err(mismatch(format!(
                "{key}: obsolete input must export false, effective {actual}"
            )));
        }
        return Ok(serde_json::json!({
            "key": key, "requested": requested, "effective": actual,
            "normalization": {
                "kind": "obsolete_input",
                "expected_effective": "0",
                "source": "PrintConfig.cpp::PrintConfigDef::handle_legacy; silent_mode default"
            }
        }));
    }
    // `PrintConfig.cpp:8681-8683`: normalize_fdm_1 floors resolution at 1um.
    if key == "resolution"
        && requested
            .as_str()
            .and_then(|value| value.parse::<f64>().ok())
            .is_some_and(|value| (0.0..0.001).contains(&value))
    {
        if actual.as_str() != Some("0.001") {
            return Err(mismatch(format!(
                "{key}: subthreshold input must export 0.001, effective {actual}"
            )));
        }
        return Ok(serde_json::json!({
            "key": key, "requested": requested, "effective": actual,
            "normalization": {
                "kind": "resolution_floor",
                "expected_effective": "0.001",
                "source": "PrintConfig.cpp::DynamicPrintConfig::normalize_fdm_1"
            }
        }));
    }
    if key == "enable_arc_fitting" {
        // PrintConfig.cpp (345efec64c7b):9886-9891: normalize only the
        // source-owned Boolean legacy tokens, never vector/scalar shapes.
        let normalized: ares_core::SliceOptions =
            serde_json::from_value(serde_json::json!({key: requested}))
                .map_err(|error| mismatch(error.to_string()))?;
        let canonical = &normalized.values()[key];
        if !value_equivalent(actual, canonical) {
            return Err(mismatch(format!(
                "{key}: requested {requested}, canonical {canonical}, effective {actual}"
            )));
        }
        return Ok(serde_json::json!({
            "key": key, "requested": requested, "effective": actual,
            "normalization": {"kind": "arc_fitting_legacy_enum", "source": "PrintConfig.cpp:9886-9891"}
        }));
    }
    if !value_equivalent(actual, requested) {
        return Err(mismatch(format!(
            "{key}: requested {requested}, effective {actual}"
        )));
    }
    Ok(serde_json::json!({"key": key, "requested": requested, "effective": actual}))
}

/// Upstream `ConfigBase::serialize` re-encodes values: integral floats
/// lose their decimal tail (`"2.0"` → `"2"`, `Config.hpp:777-781`) and
/// vectors keep per-element semantics. Compare against the canonical
/// form so a requested `"2.0"` matches an effective `"2"` without
/// weakening string-sensitive comparisons (percent tags, units).
fn value_equivalent(actual: &Value, requested: &Value) -> bool {
    match (actual, requested) {
        (Value::String(actual), Value::String(requested)) => {
            canonical_number_text(actual) == canonical_number_text(requested)
        }
        (Value::Array(actual), Value::Array(requested)) => {
            if actual.len() != requested.len()
                && requested.len() == 1
                && requested[0]
                    .as_str()
                    .is_some_and(|joined| joined.contains(','))
            {
                // A probe may send one comma-joined element (e.g. the
                // `XxY,XxY,...` POLYGON probe) while the canonical export
                // emits one element per value — flatten and retry before
                // declaring a mismatch.
                let flattened: Vec<Value> = requested[0]
                    .as_str()
                    .unwrap()
                    .split(',')
                    .map(|part| Value::String(part.to_owned()))
                    .collect();
                return actual.len() == flattened.len()
                    && actual
                        .iter()
                        .zip(flattened.iter())
                        .all(|(actual, requested)| value_equivalent(actual, requested));
            }
            actual.len() == requested.len()
                && actual
                    .iter()
                    .zip(requested.iter())
                    .all(|(actual, requested)| value_equivalent(actual, requested))
        }
        (actual, requested) => actual == requested,
    }
}

/// Normalize a numeric literal the way upstream serialization does:
/// `"2.0"` → `"2"`. Trailing zeroes are stripped only inside the
/// fractional part — `"10"` must not collapse to `"1"`. A trailing
/// percent sign is preserved while its numeric prefix is normalized
/// (`"54.717499%"` vs serialized `"54.7175%"`); other suffixed strings
/// (`"0.4mm"`) compare byte-for-byte.
fn canonical_number_text(text: &str) -> String {
    let (numeric, percent) = match text.strip_suffix('%') {
        Some(numeric) => (numeric, true),
        None => (text, false),
    };
    if numeric.parse::<f64>().is_err() {
        return text.to_owned();
    }
    let canonical = match numeric.split_once('.') {
        Some((integer, fraction)) => {
            let fraction = fraction.trim_end_matches('0');
            if fraction.is_empty() {
                integer.to_owned()
            } else {
                format!("{integer}.{fraction}")
            }
        }
        None => numeric.to_owned(),
    };
    if percent {
        format!("{canonical}%")
    } else {
        canonical
    }
}

pub(crate) fn verify(
    project: &[u8],
    key: &str,
    requested: &Value,
    dependencies: &Map<String, Value>,
) -> Result<Value, StageError> {
    let effective = settings(project)?;
    let mismatch =
        |detail| StageError::new(Stage::Application, FailureKind::ApplicationMismatch, detail);
    let expected = width(requested).map_err(mismatch)?;
    let actual_value = effective
        .get(key)
        .ok_or_else(|| mismatch(format!("missing {key}")))?;
    let actual = width(actual_value).map_err(mismatch)?;
    if expected != actual {
        return Err(mismatch(format!(
            "{key}: requested {requested}, effective {actual_value}"
        )));
    }
    for (key, expected) in dependencies {
        if effective.get(key) != Some(expected) {
            return Err(mismatch(format!(
                "dependency {key}: expected {expected}, effective {:?}",
                effective.get(key)
            )));
        }
    }
    let mut exported = Map::new();
    for key in [
        "nozzle_diameter",
        "layer_height",
        "thick_bridges",
        "thick_internal_bridges",
        "internal_solid_infill_line_width",
    ] {
        let value = effective
            .get(key)
            .ok_or_else(|| mismatch(format!("missing dependency {key}")))?;
        exported.insert(key.into(), value.clone());
    }
    Ok(
        serde_json::json!({"key": key, "requested": requested, "effective": actual_value,
        "typed": {"value": actual.value, "percent": actual.percent}, "dependencies": exported,
        "scope": "project export; object/region activation not proven"}),
    )
}
