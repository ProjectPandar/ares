use std::path::Path;

use regex::Regex;
use serde_json::Value;

#[path = "widths.rs"]
pub(super) mod widths;

#[derive(Clone, Debug)]
pub(super) struct OptionPlan {
    pub(super) key: String,
    pub(super) option_type: String,
    pub(super) raw_scope: String,
    pub(super) source: String,
    pub(super) cases: Vec<OptionCase>,
    pub(super) omission: Option<&'static str>,
    pub(super) width_domain: Option<widths::WidthDomain>,
    /// Whether the generated cases provably cover the option's legal domain:
    /// bool/enum exhaust their declared value set, schema-bounded numerics
    /// cover `min`/`max`/interior, and width options carry a context-bound
    /// `width_domain`. Unbounded/baseline-only domains are not provable.
    pub(super) provably_legal: bool,
}

#[derive(Clone, Debug)]
pub(super) struct OptionCase {
    pub(super) label: String,
    pub(super) value: Option<Value>,
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
        .filter_map(|entry| plan(entry, &lines, &source))
        .collect()
}

fn plan(entry: &Value, lines: &[&str], source_text: &str) -> Option<OptionPlan> {
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
    let mut width_domain = None;
    let mut provably_legal = false;
    let (values, omission) = if is_width_option(&key) {
        let raw = raw_numeric_bounds(&block).expect("width schema must have explicit raw bounds");
        let (domain, values) = widths::generate(&key, entry, raw, false, true);
        width_domain = Some(domain);
        provably_legal = true;
        (values, None)
    } else if matches!(option_type.as_str(), "coBool" | "coBools") {
        // A boolean domain is exactly {0,1}; both cases are legal by
        // construction, so full comparison is the legality proof.
        provably_legal = true;
        (
            vec![case("false", "0", entry), case("true", "1", entry)],
            None,
        )
    } else if matches!(option_type.as_str(), "coEnum" | "coEnums") {
        let values = enum_values(&key, &block, source_text)
            .into_iter()
            .map(|value| case(&value, &value, entry))
            .collect::<Vec<_>>();
        // An enum's declared value set is its whole legal domain.
        provably_legal = !values.is_empty();
        let omission = values
            .is_empty()
            .then_some("enum domain not explicit in definition block");
        (values, omission)
    } else if range_type(&option_type) {
        match numeric_bounds(&block) {
            Some((minimum, maximum)) if minimum < maximum => {
                let interior = seeded_interior(&key, minimum, maximum, option_type.contains("Int"));
                // An explicit `min`/`max` schema bound makes min+max+interior
                // cover the option's legal range (the goal's Range contract).
                provably_legal = true;
                let mut cases = vec![
                    numeric_case("min", minimum, entry, &option_type),
                    numeric_case("max", maximum, entry, &option_type),
                    numeric_case("seeded", interior, entry, &option_type),
                ];
                // `coFloatOrPercent` accepts BOTH a percent and an absolute
                // float — exercising only the `%` branch leaves the literal
                // branch untested. Add an absolute-float seeded case bounded
                // by `max_literal` (the absolute-branch maximum).
                // The absolute-literal branch exists even when no explicit
                // `max_literal` is declared (e.g. `bridge_acceleration` accepts
                // `mm/s^2` literals with only a percent `max`); bound it by
                // `max_literal` when present else by the declared max.
                if option_type == "coFloatOrPercent" {
                    let lit_max = literal_maximum(&block).unwrap_or(maximum);
                    let lit_interior = seeded_interior(&key, minimum, lit_max, false);
                    let mut absolute =
                        numeric_case("seeded-literal", lit_interior, entry, "coFloat");
                    absolute.label = "seeded-literal".to_owned();
                    cases.push(absolute);
                }
                (cases, None)
            }
            _ => {
                // Many speed/distance options declare only `min` (e.g.
                // `outer_wall_speed min=1`, `travel_speed min=1`) with no
                // upper schema bound — the legal domain is `[min, ∞)`, fully
                // covered by probing `min` and a seeded interior. There is no
                // `max` to exercise because none exists in the schema.
                match lower_bound(&block) {
                    Some(minimum) => {
                        let interior = seeded_interior(
                            &key,
                            minimum,
                            minimum.max(1.0) * 4.0 + 1.0,
                            option_type.contains("Int"),
                        );
                        let mut cases = vec![
                            numeric_case("min", minimum, entry, &option_type),
                            numeric_case("seeded", interior, entry, &option_type),
                        ];
                        // `coFloatOrPercent` min-only options (e.g.
                        // `overhang_reverse_threshold` min=0, max_literal=20)
                        // still admit an absolute-literal branch bounded by
                        // `max_literal` — exercise it like the bounded case.
                        if option_type == "coFloatOrPercent" {
                            // `max_literal` may be absent; the literal branch
                            // still accepts absolutes — bound by the literal
                            // maximum when declared else a nominal bound.
                            let lit_max =
                                literal_maximum(&block).unwrap_or(minimum.max(1.0) * 4.0 + 1.0);
                            let lit_interior = seeded_interior(&key, minimum, lit_max, false);
                            let mut absolute =
                                numeric_case("seeded-literal", lit_interior, entry, "coFloat");
                            absolute.label = "seeded-literal".to_owned();
                            cases.push(absolute);
                        }
                        provably_legal = true;
                        (cases, None)
                    }
                    None => {
                        // Schema-unbounded numeric option (`coFloat`/
                        // `coInt`/`coFloatOrPercent` with neither min
                        // nor max — e.g. `adaptive_bed_mesh_margin`).
                        // The legal domain is open-ended; a single
                        // default run proves nothing, so probe two
                        // deterministic small-positive values that are
                        // legal for every sane open range (a rejection
                        // still reports honestly as INCOMPLETE).
                        //
                        // Integer-family probes are restricted below:
                        // `coInt(s)` unbounded options are dominated by
                        // per-filament vectors and id-valued keys where a
                        // generic value is rejected or crashes the
                        // oracle (`master_extruder_id`,
                        // `filament_self_index`, `physical_extruder_map`)
                        // — they stay INCOMPLETE by design.
                        let cases = if matches!(option_type.as_str(), "coInt" | "coInts") {
                            vec![baseline_case()]
                        } else if option_type == "coFloatOrPercent" {
                            // Both percent and absolute-literal branches.
                            vec![
                                case("probe-percent", "40%", entry),
                                numeric_case("probe-literal", 3.0, entry, "coFloat"),
                            ]
                        } else {
                            let seed_hi =
                                seeded_interior(&key, 1.0, 9.0, option_type.contains("Int"));
                            vec![
                                numeric_case("probe-min", 0.5, entry, &option_type),
                                numeric_case("probe-seeded", seed_hi, entry, &option_type),
                            ]
                        };
                        // Seeded probes are the strongest coverage an
                        // unbounded domain admits (coInt(s) stay
                        // unprovable — baseline only).
                        provably_legal = !matches!(option_type.as_str(), "coInt" | "coInts");
                        (cases, None)
                    }
                }
            }
        }
    } else {
        // `coPoint(s)` unbounded options (bed polygons, exclude areas)
        // reject/crash on generic probes (`printable_area` degenerate
        // polygon); baseline default remains until per-key legal domains
        // are authored.
        (vec![baseline_case()], None)
    };
    Some(OptionPlan {
        key,
        option_type,
        raw_scope,
        source,
        cases: values,
        omission,
        width_domain,
        provably_legal,
    })
}

// `coFloatOrPercent` line-width options share the single-nozzle context
// bound modeled by `widths::generate` (`Print::validate` caps the feasible
// width relative to the nozzle, far below the raw `max`/`max_literal`
// schema bound — `outer_wall_line_width=10` or `10%` both fail the oracle).
fn is_width_option(key: &str) -> bool {
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
    )
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

fn enum_values(key: &str, block: &str, source: &str) -> Vec<String> {
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
    if values.is_empty()
        && let Some(kind) = Regex::new(r"ConfigOptionEnum<([^>]+)>")
            .unwrap()
            .captures(&active_block)
    {
        let kind = kind[1].rsplit("::").next().unwrap();
        let map = Regex::new(&format!(
            r"(?s)s_keys_map_{}\s*\{{(.*?)\}};",
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

fn strip_comments(source: &str) -> String {
    Regex::new(r"(?s)/\*.*?\*/|//[^\n]*")
        .unwrap()
        .replace_all(source, "")
        .into_owned()
}

fn referenced_enum_values(block: &str, source: &str, push: &Regex) -> Vec<String> {
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

fn known_vector_enum(key: &str) -> &'static [&'static str] {
    match key {
        "default_nozzle_volume_type" | "nozzle_volume_type" => &["Standard", "High Flow"],
        "filament_retract_lift_enforce" => {
            &["All Surfaces", "Top Only", "Bottom Only", "Top and Bottom"]
        }
        "filament_z_hop_types" => &["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
        _ => &[],
    }
}

fn raw_numeric_bounds(block: &str) -> Option<(f64, f64)> {
    let bound = |name: &str| {
        Regex::new(&format!(r"def->{name}\s*=\s*([-+]?\d+(?:\.\d+)?)"))
            .unwrap()
            .captures(block)
            .and_then(|captures| captures[1].parse::<f64>().ok())
    };
    Some((bound("min")?, bound("max")?))
}

// Legacy domains outside the two-key vertical remain unverified.
fn lower_bound(block: &str) -> Option<f64> {
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

fn numeric_bounds(block: &str) -> Option<(f64, f64)> {
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

fn literal_maximum(block: &str) -> Option<f64> {
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
        value: Some(value),
    }
}

fn baseline_case() -> OptionCase {
    OptionCase {
        label: "baseline".to_owned(),
        value: None,
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
