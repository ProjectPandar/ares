use std::path::Path;

use regex::Regex;
use serde_json::Value;

#[derive(Clone, Debug)]
pub(super) struct OptionPlan {
    pub(super) key: String,
    pub(super) option_type: String,
    pub(super) raw_scope: String,
    pub(super) source: String,
    pub(super) cases: Vec<OptionCase>,
    pub(super) omission: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub(super) struct OptionCase {
    pub(super) label: String,
    pub(super) value: Value,
}

pub(super) fn load(repo: &Path) -> Vec<OptionPlan> {
    let inventory: Value = serde_json::from_str(
        &std::fs::read_to_string(repo.join("tests/ksr_fdmtest_v4/options-v242.json")).unwrap(),
    )
    .unwrap();
    let source =
        std::fs::read_to_string(repo.join("OrcaSlicer/src/libslic3r/PrintConfig.cpp")).unwrap();
    let lines = source.lines().collect::<Vec<_>>();
    inventory
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| plan(entry, &lines))
        .collect()
}

fn plan(entry: &Value, lines: &[&str]) -> Option<OptionPlan> {
    let key = entry.get("key")?.as_str()?.to_owned();
    let option_type = entry.get("option_type")?.as_str()?.to_owned();
    let raw_scope = entry.get("raw_scope")?.as_str()?.to_owned();
    if option_type == "Metadata" {
        return None;
    }
    let definition = entry.get("upstream_definition")?;
    let line = definition.get("line")?.as_u64()? as usize;
    let source = format!("{}:{}", definition.get("path")?.as_str()?, line);
    let block = definition_block(lines, &key, line);
    let (values, omission) = if matches!(option_type.as_str(), "coBool" | "coBools") {
        (
            vec![case("false", "0", entry), case("true", "1", entry)],
            None,
        )
    } else if matches!(option_type.as_str(), "coEnum" | "coEnums") {
        let values = enum_values(&block)
            .into_iter()
            .map(|value| case(&value, &value, entry))
            .collect::<Vec<_>>();
        let omission = values
            .is_empty()
            .then_some("enum domain not explicit in definition block");
        (values, omission)
    } else if range_type(&option_type) {
        match numeric_bounds(&block) {
            Some((minimum, maximum)) if minimum < maximum => {
                let interior = seeded_interior(&key, minimum, maximum, option_type.contains("Int"));
                (
                    vec![
                        numeric_case("min", minimum, entry, &option_type),
                        numeric_case("max", maximum, entry, &option_type),
                        numeric_case("seeded", interior, entry, &option_type),
                    ],
                    None,
                )
            }
            _ => (
                Vec::new(),
                Some("bounded min/max not explicit in definition block"),
            ),
        }
    } else {
        (Vec::new(), Some("non boolean/enum/range option"))
    };
    Some(OptionPlan {
        key,
        option_type,
        raw_scope,
        source,
        cases: values,
        omission,
    })
}

fn definition_block(lines: &[&str], key: &str, reported_line: usize) -> String {
    let needle = format!("add(\"{key}\"");
    let lower = reported_line.saturating_sub(20);
    let upper = (reported_line + 20).min(lines.len());
    let start = (lower..upper)
        .find(|index| lines[*index].contains(&needle))
        .unwrap_or_else(|| reported_line.saturating_sub(1));
    let end = (start + 1..lines.len())
        .find(|index| lines[*index].contains("this->add(\"") || lines[*index].contains("add(\""))
        .unwrap_or(lines.len());
    lines[start..end.min(start + 160)].join("\n")
}

fn enum_values(block: &str) -> Vec<String> {
    let push = Regex::new(r#"enum_values\.push_back\("([^"]+)"\)"#).unwrap();
    let quoted = Regex::new(r#""([^"]+)""#).unwrap();
    let mut values = push
        .captures_iter(block)
        .map(|captures| captures[1].to_owned())
        .collect::<Vec<_>>();
    if values.is_empty()
        && let Some(start) = block.find("enum_values = {")
        && let Some(end) = block[start..].find("};")
    {
        values.extend(
            quoted
                .captures_iter(&block[start..start + end])
                .map(|captures| captures[1].to_owned()),
        );
    }
    values.sort();
    values.dedup();
    values
}

fn numeric_bounds(block: &str) -> Option<(f64, f64)> {
    let bound = |name: &str| {
        Regex::new(&format!(r"def->{name}\s*=\s*([-+]?\d+(?:\.\d+)?)"))
            .unwrap()
            .captures(block)
            .and_then(|captures| captures[1].parse().ok())
    };
    Some((bound("min")?, bound("max")?))
}

fn seeded_interior(key: &str, minimum: f64, maximum: f64, integer: bool) -> f64 {
    let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    });
    let ratio = 0.2 + (hash % 600_001) as f64 / 1_000_000.0;
    let value = minimum + ratio * (maximum - minimum);
    if integer {
        value.round().clamp(minimum + 1.0, maximum - 1.0)
    } else {
        value
    }
}

fn numeric_case(label: &str, value: f64, entry: &Value, option_type: &str) -> OptionCase {
    let rendered = if option_type.contains("Int") {
        format!("{value:.0}")
    } else if option_type.contains("Percent") {
        format!("{value}%")
    } else {
        value.to_string()
    };
    case(label, &rendered, entry)
}

fn case(label: &str, value: &str, entry: &Value) -> OptionCase {
    let scalar = Value::String(value.to_owned());
    let value = if entry.get("wire_shape").and_then(Value::as_str) == Some("array") {
        Value::Array(vec![scalar])
    } else {
        scalar
    };
    OptionCase {
        label: label.to_owned(),
        value,
    }
}

fn range_type(option_type: &str) -> bool {
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
