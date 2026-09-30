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
    let (values, mut omission) = if let Some(authored) = authored_probes(&key) {
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
                if key == "enable_long_retraction_when_cut" {
                    "named modes covered; unbounded coInt endpoints remain unverified"
                } else {
                    "authored probes sample the legal interior; schema bounds rejected by upstream validation"
                },
            ),
        )
    } else if is_width_option(&key) {
        let locked = matches!(
            key.as_str(),
            "skin_infill_line_width" | "skeleton_infill_line_width"
        );
        let raw = if locked {
            // These definitions inherit ConfigOptionDef's default FLT_MAX ceiling.
            (0.0, f64::from(f32::MAX))
        } else {
            raw_numeric_bounds(&block).expect("width schema must have explicit raw bounds")
        };
        let (domain, values) = widths::generate(&key, entry, raw, false, true);
        width_domain = Some(domain);
        provably_legal = !locked;
        (
            values,
            locked.then_some(
                "single-nozzle context covered; global range endpoints remain unverified",
            ),
        )
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
        let bounds = if matches!(
            key.as_str(),
            "temperature_vitrification" | "filament_printable"
        ) {
            // ConfigOptionInts reserves INT_MAX as nil, even when non-nullable.
            Some((f64::from(i32::MIN), f64::from(i32::MAX - 1)))
        } else if key == "skirt_height" {
            // `PrintConfig.cpp:10422-10424` rejects negatives at validation,
            // although the option definition only declares the upper bound.
            Some((0.0, upper_bound(&block).expect("skirt height ceiling")))
        } else if key == "hot_plate_temp_initial_layer" {
            // `ConfigOptionInts` stores signed 32-bit values; the default
            // schema min (-FLT_MAX) is below its representable domain.
            Some((
                f64::from(i32::MIN),
                upper_bound(&block).expect("hot plate ceiling"),
            ))
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
                if key == "temperature_vitrification" {
                    cases.push(case("probe", "110", entry));
                }
                if key == "filament_printable" {
                    cases.extend(["0", "1", "2", "3"].map(|value| case(value, value, entry)));
                }
                if key == "filament_retraction_distances_when_cut" {
                    // Generated nullable override: nil selects the machine distance.
                    cases.push(case("nil", "nil", entry));
                }
                // Both units use the full schema range, not the GUI's
                // max_literal warning threshold (Config.hpp:2481-2482).
                if option_type == "coFloatOrPercent" {
                    for (label, value) in [
                        ("min-literal", minimum),
                        ("max-literal", maximum),
                        (
                            "seeded-literal",
                            seeded_interior(&key, minimum, maximum, false),
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
                // Retain local comparison probes, but absent assignments inherit
                // ConfigOptionDef's finite defaults. Unknown expressions also
                // leave endpoint and full-range seeded coverage unproved.
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
                        (
                            vec![
                                numeric_case("max", maximum, entry, &option_type),
                                numeric_case("seeded", interior, entry, &option_type),
                            ],
                            Some(incomplete_numeric_domain(&block)),
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
                        // still admit an absolute-literal branch. `max_literal`
                        // only supplies a local probe scale, not a legal bound.
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
                        (cases, Some(incomplete_numeric_domain(&block)))
                    }
                    (None, None) => {
                        // Small local probes do not cover implicit endpoints.
                        // Integer ID/vector options retain baseline-only probes
                        // because generic values can crash the original slicer.
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
                        (cases, Some(incomplete_numeric_domain(&block)))
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
    // Config.hpp:2475-2479 supplies FLT_MAX when resolution omits its max.
    if key == "resolution" {
        provably_legal = false;
        omission = Some("implicit FLT_MAX ceiling and full-range interior remain unverified");
    }
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
