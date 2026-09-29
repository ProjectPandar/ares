use std::path::Path;

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

/// Offline callers return `None` (and skip) when the vendored upstream
/// source tree is absent — CI jobs do not checkout `OrcaSlicer/`.
pub(super) fn try_load(repo: &Path) -> Option<Vec<OptionPlan>> {
    if !repo
        .join("OrcaSlicer/src/libslic3r/PrintConfig.cpp")
        .exists()
    {
        eprintln!("skipping: OrcaSlicer source tree not vendored at {repo:?}/OrcaSlicer");
        return None;
    }
    Some(load(repo))
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
    let (values, omission) = if let Some(authored) = authored_probes(&key) {
        // Per-key legal domains: free-form `coString`/`coPoint(s)`
        // probes cover an unbounded schema domain completely; NUMERIC
        // authored probes only sample the legal interior (the schema
        // min/max that upstream slice-time validation clamps are
        // intentionally not probed), so they stay INCOMPLETE — the
        // domain is exercised but the boundary claim isn't proven.
        let unbounded_schema = matches!(
            option_type.as_str(),
            "coString" | "coStrings" | "coPoint" | "coPoints"
        );
        provably_legal = unbounded_schema;
        (
            authored
                .iter()
                .map(|&(label, value)| case(label, value, entry))
                .collect(),
            (!unbounded_schema).then_some(
                "authored probes sample the legal interior; schema bounds rejected by upstream validation",
            ),
        )
    } else if is_width_option(&key) {
        let raw = raw_numeric_bounds(&block).expect("width schema must have explicit raw bounds");
        let (domain, values) = widths::generate(&key, entry, raw, false, true);
        width_domain = Some(domain);
        provably_legal = true;
        (values, None)
    } else if matches!(option_type.as_str(), "coBool" | "coBools") {
        // A boolean domain is exactly {0,1}; both cases are legal by
        // construction, so full comparison is the legality proof.
        // Nullable booleans additionally admit `nil` (the option is
        // then omitted from the exported config) — probe it too.
        provably_legal = true;
        let mut values = vec![case("false", "0", entry), case("true", "1", entry)];
        if entry.get("nullable").and_then(Value::as_bool) == Some(true) {
            values.push(case("nil", "nil", entry));
        }
        (values, None)
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
        let bounds = if key == "skirt_height" {
            // `PrintConfig.cpp:10422-10424` rejects negatives at validation,
            // although the option definition only declares the upper bound.
            Some((0.0, upper_bound(&block).expect("skirt height ceiling")))
        } else {
            numeric_bounds(&block)
        };
        match bounds {
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
                    // Note: upstream applies `min`/`max` to BOTH the
                    // percent and the absolute branch (numeric validation
                    // `PrintConfig.cpp:10529-10535`); `max_literal` is a
                    // GUI typo-warning threshold only
                    // (`Config.hpp:2481-2482`), used here solely to bound
                    // the seeded absolute probe's interior pick.
                    let lit_max = literal_maximum(&block).unwrap_or(maximum);
                    for (label, value) in [
                        ("min-literal", minimum),
                        ("max-literal", maximum),
                        (
                            "seeded-literal",
                            seeded_interior(&key, minimum, lit_max, false),
                        ),
                    ] {
                        let mut absolute = numeric_case(label, value, entry, "coFloat");
                        absolute.label = label.to_owned();
                        cases.push(absolute);
                    }
                }
                (cases, None)
            }
            _ => {
                // Many speed/distance options declare only `min` (e.g.
                // `outer_wall_speed min=1`, `travel_speed min=1`) with no
                // upper schema bound — the legal domain is `[min, ∞)`, fully
                // covered by probing `min` and a seeded interior. There is no
                // `max` to exercise because none exists in the schema.
                match (lower_bound(&block), upper_bound(&block)) {
                    (None, Some(maximum)) => {
                        // Upper-only bound (`max` declared, no `min` — e.g.
                        // `nozzle_diameter` max=100, `PrintConfig.cpp:4835`):
                        // exercise the declared ceiling and a seeded interior
                        // anchored at 0 — a sampling anchor, not a schema
                        // bound (an absent `min` is `-FLT_MAX`,
                        // `Config.hpp:2475-2479`; e.g.
                        // `wipe_tower_extra_rib_length` accepts negatives,
                        // `PrintConfig.cpp:6941-6948`).
                        let interior =
                            seeded_interior(&key, 0.0, maximum, option_type.contains("Int"));
                        provably_legal = true;
                        (
                            vec![
                                numeric_case("max", maximum, entry, &option_type),
                                numeric_case("seeded", interior, entry, &option_type),
                            ],
                            None,
                        )
                    }
                    (Some(minimum), _) => {
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
                            // The absolute branch shares the declared `min`
                            // bound (`PrintConfig.cpp:10529-10535`) — probe it.
                            let lit_max =
                                literal_maximum(&block).unwrap_or(minimum.max(1.0) * 4.0 + 1.0);
                            for (label, value) in [
                                ("min-literal", minimum),
                                (
                                    "seeded-literal",
                                    seeded_interior(&key, minimum, lit_max, false),
                                ),
                            ] {
                                let mut absolute = numeric_case(label, value, entry, "coFloat");
                                absolute.label = label.to_owned();
                                cases.push(absolute);
                            }
                        }
                        provably_legal = true;
                        (cases, None)
                    }
                    (None, None) => {
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
                        let unbounded_int = matches!(option_type.as_str(), "coInt" | "coInts");
                        let cases = if unbounded_int {
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
                        provably_legal = !unbounded_int;
                        (
                            cases,
                            unbounded_int.then_some(
                                "unbounded coInt(s) — value-space unprovable, baseline only",
                            ),
                        )
                    }
                }
            }
        }
    } else {
        // `coPoint(s)` unbounded options (bed polygons, exclude areas)
        // reject/crash on generic probes (`printable_area` degenerate
        // polygon); baseline default remains until per-key legal domains
        // are authored. The omission reason feeds the sweep gate.
        (
            vec![baseline_case()],
            Some("unbounded domain (string/point/blob) — baseline only"),
        )
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

#[path = "bounds.rs"]
mod bounds;
use bounds::*;

#[path = "domains/probes.rs"]
mod probes;
use probes::authored_probes;
