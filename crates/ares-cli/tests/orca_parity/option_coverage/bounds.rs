use regex::Regex;
use serde_json::Value;

use super::OptionCase;

// `coFloatOrPercent` line-width options share the single-nozzle context
// bound modeled by `widths::generate` (`Print::validate` caps the feasible
// width relative to the nozzle, far below the raw `max`/`max_literal`
// schema bound — `outer_wall_line_width=10` or `10%` both fail the oracle).
pub(super) fn is_width_option(key: &str) -> bool {
    matches!(
        key,
        "line_width"
            | "bridge_line_width"
            | "outer_wall_line_width"
            | "inner_wall_line_width"
            | "initial_layer_line_width"
            | "sparse_infill_line_width"
            | "internal_solid_infill_line_width"
            | "support_line_width"
            | "top_surface_line_width"
            | "skin_infill_line_width"
            | "skeleton_infill_line_width"
    )
}

/// Upstream generates `filament_*` extruder-override definitions via
/// `add_nullable(opt_key, base.type)` (`PrintConfig.cpp:7288-7302`),
/// copying min/max/enum metadata from the base key — there is no
/// literal `add("filament_retraction_distances_when_cut"`. Resolve the
/// base key for such generated options so their bounds come from the
/// real definition instead of unrelated neighbouring source.
pub(super) fn definition_block(lines: &[&str], key: &str, reported_line: usize) -> String {
    let needle = format!("add(\"{key}\"");
    let lower = reported_line.saturating_sub(20);
    let upper = (reported_line + 20).min(lines.len());
    let mut start = (lower..upper).find(|index| lines[*index].contains(&needle));
    if start.is_none() {
        // Generated filament_* overrides have no literal `add("filament_..."`
        // definition — the inventory points at the key list, not a `def`.
        // Fall back to the base key's `add("base"` definition anywhere in
        // the file (`PrintConfig.cpp:7288-7302` copies its min/max/enums).
        if let Some(base) = key
            .strip_prefix("filament_")
            .filter(|base| !base.is_empty())
        {
            let base_needle = format!("add(\"{base}\"");
            if let Some(index) = lines.iter().position(|line| line.contains(&base_needle)) {
                start = Some(index);
            }
        }
    }
    let start = start.unwrap_or_else(|| reported_line.saturating_sub(1));
    let end = (start + 1..lines.len())
        .find(|index| lines[*index].contains("this->add(\"") || lines[*index].contains("add(\""))
        .unwrap_or(lines.len());
    lines[start..end.min(start + 160)].join("\n")
}

pub(super) fn enum_values(key: &str, block: &str, source: &str) -> Vec<String> {
    let push = Regex::new(r#"enum_values\.(?:push_back|emplace_back)\("([^"]+)"\)"#).unwrap();
    let quoted = Regex::new(r#""([^"]+)""#).unwrap();
    let active_block = strip_comments(block);
    let mut values = push
        .captures_iter(&active_block)
        .map(|captures| captures[1].to_owned())
        .collect::<Vec<_>>();
    if values.is_empty() {
        let assignment = Regex::new(r"enum_values\s*=\s*\{").unwrap();
        if let Some(found) = assignment.find(&active_block)
            && let Some(end) = active_block[found.end()..].find("};")
        {
            values.extend(
                quoted
                    .captures_iter(&active_block[found.end()..found.end() + end])
                    .map(|captures| captures[1].to_owned()),
            );
        }
    }
    if values.is_empty() {
        values.extend(referenced_enum_values(&active_block, source, &push));
    }
    // PrintConfig.cpp:579-584: Default is UI-only, absent from the wire map.
    if key == "filament_map_mode" {
        values.clear();
    }
    if values.is_empty()
        && let Some(kind) = Regex::new(r"ConfigOptionEnum<([^>]+)>")
            .unwrap()
            .captures(&active_block)
    {
        let kind = kind[1].rsplit("::").next().unwrap();
        let map = Regex::new(&format!(
            r"(?s)s_keys_map_{}\s*(?:=\s*)?\{{(.*?)\}};",
            regex::escape(kind)
        ))
        .unwrap();
        if let Some(body) = map.captures(source) {
            values.extend(
                quoted
                    .captures_iter(&body[1])
                    .map(|captures| captures[1].to_owned()),
            );
        }
    }
    if values.is_empty() {
        values.extend(
            known_vector_enum(key)
                .iter()
                .map(|value| (*value).to_owned()),
        );
    }
    values.sort();
    values.dedup();
    values
}

pub(super) fn strip_comments(source: &str) -> String {
    Regex::new(r"(?s)/\*.*?\*/|//[^\n]*")
        .unwrap()
        .replace_all(source, "")
        .into_owned()
}

pub(super) fn referenced_enum_values(block: &str, source: &str, push: &Regex) -> Vec<String> {
    let Some(reference) = Regex::new(r"enum_values\s*=\s*(\w+)->enum_values")
        .unwrap()
        .captures(block)
        .map(|captures| captures[1].to_owned())
    else {
        return Vec::new();
    };
    let marker = Regex::new(&format!(
        r"(?:auto\s+)?{}\s*=\s*def\s*=\s*this->add\(",
        regex::escape(&reference)
    ))
    .unwrap();
    let Some(start) = marker.find(source).map(|found| found.start()) else {
        return Vec::new();
    };
    let tail = &source[start..];
    let end = tail[marker.find(tail).unwrap().end()..]
        .find("this->add(\"")
        .map_or(tail.len(), |offset| {
            marker.find(tail).unwrap().end() + offset
        });
    let active = strip_comments(&tail[..end]);
    push.captures_iter(&active)
        .map(|captures| captures[1].to_owned())
        .collect()
}

pub(super) fn known_vector_enum(key: &str) -> &'static [&'static str] {
    match key {
        "default_nozzle_volume_type" | "nozzle_volume_type" => &["Standard", "High Flow"],
        "filament_retract_lift_enforce" => {
            &["All Surfaces", "Top Only", "Bottom Only", "Top and Bottom"]
        }
        "filament_z_hop_types" => &["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
        _ => &[],
    }
}

pub(super) fn raw_numeric_bounds(block: &str) -> Option<(f64, f64)> {
    let bound = |name: &str| {
        Regex::new(&format!(r"def->{name}\s*=\s*([-+]?\d+(?:\.\d+)?)"))
            .unwrap()
            .captures(block)
            .and_then(|captures| captures[1].parse::<f64>().ok())
    };
    Some((bound("min")?, bound("max")?))
}

// Legacy domains outside the two-key vertical remain unverified.
pub(super) fn lower_bound(block: &str) -> Option<f64> {
    Regex::new(r"def->min\s*=\s*([-+]?\d+(?:\.\d+)?|-?max_temp)")
        .unwrap()
        .captures(block)
        .and_then(|captures| {
            let token = &captures[1];
            if token == "max_temp" {
                Some(1500.0)
            } else if token == "-max_temp" {
                Some(-1500.0)
            } else {
                token.parse::<f64>().ok()
            }
        })
}

pub(super) fn upper_bound(block: &str) -> Option<f64> {
    Regex::new(r"def->max\s*=\s*([-+]?\d+(?:\.\d+)?|-?max_temp)")
        .unwrap()
        .captures(block)
        .and_then(|captures| {
            let token = &captures[1];
            if token == "max_temp" {
                Some(1500.0)
            } else if token == "-max_temp" {
                Some(-1500.0)
            } else {
                token.parse::<f64>().ok()
            }
        })
}

pub(super) fn numeric_bounds(block: &str) -> Option<(f64, f64)> {
    let bound = |name: &str| {
        // `def->max = <literal>` OR `def->max = max_temp` (the 1500 constant
        // upstream defines at `PrintConfig.cpp:939` for every temperature).
        Regex::new(&format!(
            r"def->{name}\s*=\s*([-+]?\d+(?:\.\d+)?|-?max_temp)"
        ))
        .unwrap()
        .captures(block)
        .and_then(|captures| {
            let token = &captures[1];
            if token == "max_temp" {
                Some(1500.0)
            } else if token == "-max_temp" {
                Some(-1500.0)
            } else {
                token.parse::<f64>().ok()
            }
        })
    };
    // `def->max` is the PERCENT maximum; `def->max_literal` is the separate
    // maximum for the absolute-float branch of `coFloatOrPercent` — the two
    // are independent bounds, not a single ceiling. Return the percent bound
    // untouched; the literal bound is surfaced separately for dual-mode opts.
    let minimum = bound("min")?;
    let maximum = bound("max")?;
    Some((minimum, maximum))
}

pub(super) fn literal_maximum(block: &str) -> Option<f64> {
    Regex::new(r"def->max_literal\s*=\s*([-+]?\d+(?:\.\d+)?|-?max_temp)")
        .unwrap()
        .captures(block)
        .and_then(|captures| {
            let token = &captures[1];
            if token == "max_temp" {
                Some(1500.0)
            } else if token == "-max_temp" {
                Some(-1500.0)
            } else {
                token.parse::<f64>().ok()
            }
        })
}

pub(super) fn seeded_interior(key: &str, minimum: f64, maximum: f64, integer: bool) -> f64 {
    let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    });
    let ratio = 0.2 + (hash % 600_001) as f64 / 1_000_000.0;
    let value = minimum + ratio * (maximum - minimum);
    if integer {
        value.round().clamp(minimum + 1.0, maximum - 1.0)
    } else {
        // Quantize to the milligrid upstream `opt_serialize` writes (3
        // decimals): raw `ratio`-scaled floats carry binary noise
        // (`590.6709999999999`) that serializes differently (`590.671`),
        // making the application check report a false mismatch.
        (value * 1000.0).round() / 1000.0
    }
}

pub(super) fn numeric_case(
    label: &str,
    value: f64,
    entry: &Value,
    option_type: &str,
) -> OptionCase {
    let rendered = if option_type.contains("Int") {
        format!("{value:.0}")
    } else if option_type.contains("Percent") {
        format!("{value}%")
    } else {
        value.to_string()
    };
    case(label, &rendered, entry)
}

pub(super) fn case(label: &str, value: &str, entry: &Value) -> OptionCase {
    let scalar = Value::String(value.to_owned());
    let value = if entry.get("wire_shape").and_then(Value::as_str) == Some("array") {
        Value::Array(vec![scalar])
    } else {
        scalar
    };
    OptionCase {
        label: label.to_owned(),
        value: Some(value),
    }
}

pub(super) fn baseline_case() -> OptionCase {
    OptionCase {
        label: "baseline".to_owned(),
        value: None,
    }
}

pub(super) fn range_type(option_type: &str) -> bool {
    matches!(
        option_type,
        "coFloat"
            | "coFloats"
            | "coInt"
            | "coInts"
            | "coPercent"
            | "coPercents"
            | "coFloatOrPercent"
    )
}
