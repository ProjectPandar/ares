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
    } else if let Some(authored) = authored_probes(&key) {
        // Per-key legal domains: `coString`/`coPoint`/`coPoints` keys whose
        // upstream schema accepts arbitrary content (g-code templates,
        // notes, polygon lists) get concrete probes rather than an
        // unexercised baseline.
        provably_legal = true;
        (
            authored
                .iter()
                .map(|&(label, value)| case(label, value, entry))
                .collect(),
            None,
        )
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

/// Per-key authored legal probes for schema-unbounded string/point
/// domains. Every listed value is accepted by upstream validation
/// (free-form g-code templates, note fields, `XxY` polygon lists), so the
/// probes exercise real emit paths on both sides rather than a
/// value-less baseline.
fn authored_probes(key: &str) -> Option<&'static [(&'static str, &'static str)]> {
    // Free-form g-code templates: any text is legal input; a non-empty
    // template with a placeholder exercises the placeholder engine.
    // `{layer_num}` is only defined once layer context exists
    // (`GCode.cpp:4669` sets it per layer); printer-scoped templates
    // rendered outside layer context (`file_start_gcode`, pauses,
    // custom/object transitions) throw on unknown variables
    // (`PlaceholderParser.cpp:997-1000`), so they get a literal probe.
    const TEMPLATE: &[(&str, &str)] = &[
        ("probe-empty", ""),
        ("probe-text", ";ARES_PROBE {layer_num}"),
    ];
    const TEMPLATE_LITERAL: &[(&str, &str)] = &[("probe-empty", ""), ("probe-text", ";ARES_PROBE")];
    const POLYGON: &[(&str, &str)] = &[("probe-rect", "10x10,20x10,20x20,10x20")];
    const POINT: &[(&str, &str)] = &[("probe", "5x5")];
    // Free-form metadata/note strings — content never alters toolpaths;
    // a distinct probe exercises profile round-trip equality.
    const NOTE: &[(&str, &str)] = &[("probe", "ares-parity-probe")];
    let probes = match key {
        // G-code template strings (`*_gcode`) — free-form by schema.
        // Layer-context templates — `{layer_num}` is defined here.
        // `before_layer_change_gcode` must keep the baseline `G92 E0`
        // under relative extruder addressing, so no empty probe.
        "before_layer_change_gcode" => &[("probe-text", "G92 E0\n;ARES_PROBE {layer_num}")],
        "layer_change_gcode" => TEMPLATE,
        // Contexts without layer variables get a literal probe only.
        "machine_pause_gcode"
        | "printing_by_object_gcode"
        | "change_filament_gcode"
        | "change_extrusion_role_gcode"
        | "process_change_extrusion_role_gcode"
        | "filament_change_extrusion_role_gcode"
        | "template_custom_gcode"
        | "wrapping_detection_gcode"
        | "file_start_gcode" => TEMPLATE_LITERAL,
        // `time_lapse_gcode` needs the same probe but is wrapped in
        // `;TIMELAPSE` handling — still free-form.
        "time_lapse_gcode" => TEMPLATE_LITERAL,
        // Free-form note/colour/profile metadata strings.
        "filament_notes"
        | "printer_notes"
        | "model_notes"
        | "filament_colour"
        | "default_filament_colour"
        | "extruder_colour"
        | "filament_multi_colour"
        | "filament_colour_type"
        | "default_bed_type"
        | "default_print_profile"
        | "default_filament_profile"
        | "bed_custom_texture"
        | "bed_custom_model" => NOTE,
        // Polygon point lists (`XxY` pairs) — a legal small rectangle.
        // `wrapping_exclude_area` rejected by the oracle (orca rejects the
        // rect probe upstream), so it keeps its unexercised baseline.
        "bed_exclude_area" | "head_wrap_detect_zone" | "start_end_points" => POLYGON,
        // `extruder_offset`: one offset per extruder — a shifted probe
        // stays legal for a single-extruder profile.
        "extruder_offset" => &[("probe-zero", "0x0"), ("probe-shift", "1x0.5")],
        // `extra_solid_infills` accepts `N`, `N#K`, or `1,7,9` layer
        // patterns (`PrintConfig.cpp:2987-2991`); `2` (every 2nd layer
        // solid) is a legal probe that changes toolpaths on both sides.
        "extra_solid_infills" => &[("probe-2", "2")],
        // Single `XxY` points.
        "bed_mesh_min" | "bed_mesh_max" | "bed_mesh_probe_distance" => POINT,
        // `best_object_pos` is normalized `[0,1]` (`PrintConfig.cpp:3818-3822`).
        "best_object_pos" => &[("probe", "0.25x0.75")],
        _ => return None,
    };
    Some(probes)
}

#[path = "bounds.rs"]
mod bounds;
use bounds::*;
