use serde_json::Value;

use super::{OptionCase, bounds::*};

pub(super) fn generate(
    entry: &Value,
    key: &str,
    option_type: &str,
    block: &str,
) -> (Vec<OptionCase>, Option<&'static str>, bool) {
    let bounds = if key == "skirt_height" {
        // `PrintConfig.cpp:10422-10424` rejects negatives at validation,
        // although the option definition only declares the upper bound.
        Some((0.0, upper_bound(block).expect("skirt height ceiling")))
    } else {
        numeric_bounds(block)
    };
    match bounds {
        Some((minimum, maximum)) if minimum < maximum => {
            let interior = seeded_interior(key, minimum, maximum, option_type.contains("Int"));
            // Declared endpoints plus existing interior samples, not full-range coverage.
            let mut cases = vec![
                numeric_case("min", minimum, entry, option_type),
                numeric_case("max", maximum, entry, option_type),
                numeric_case("seeded", interior, entry, option_type),
            ];
            if key == "filament_retraction_distances_when_cut" {
                // Generated nullable override: nil selects the machine distance.
                cases.push(case("nil", "nil", entry));
            }
            // Both units use the full schema range, not the GUI's
            // max_literal warning threshold (Config.hpp:2481-2482).
            if option_type == "coFloatOrPercent" {
                extend_literals(
                    &mut cases,
                    entry,
                    [
                        ("min-literal", minimum),
                        ("max-literal", maximum),
                        (
                            "seeded-literal",
                            seeded_interior(key, minimum, maximum, false),
                        ),
                    ],
                );
            }
            (cases, None, true)
        }
        _ => {
            let (mut cases, mut omission) = incomplete_cases(entry, key, option_type, block);
            match key {
                "temperature_vitrification" => {
                    cases = vec![case("probe", "110", entry)];
                    omission = None;
                }
                "filament_printable" => {
                    cases = ["0", "1", "2", "3"]
                        .map(|value| case(value, value, entry))
                        .to_vec();
                    omission = None;
                }
                "slow_down_layers" => {
                    cases.extend(["1", "4"].map(|value| case(value, value, entry)))
                }
                _ => {}
            }
            if key == "tree_support_top_rate" {
                let minimum = lower_bound(block).expect("branch density minimum");
                cases.push(numeric_case(
                    "probe",
                    seeded_interior(key, minimum, minimum.max(1.0) * 4.0 + 1.0, false),
                    entry,
                    option_type,
                ));
            }
            let complete = omission.is_none() && cases.iter().all(|case| case.value.is_some());
            (cases, omission, complete)
        }
    }
}

fn incomplete_cases(
    entry: &Value,
    key: &str,
    option_type: &str,
    block: &str,
) -> (Vec<OptionCase>, Option<&'static str>) {
    // An absent source assignment is unbounded, not an endpoint obligation.
    // Unknown expressions remain unresolved; local samples do not resolve them.
    let omission = [numeric_bound(block, "min"), numeric_bound(block, "max")]
        .iter()
        .any(|bound| matches!(bound, Some(Err(()))))
        .then_some(incomplete_numeric_domain(block));
    match (lower_bound(block), upper_bound(block)) {
        (None, Some(maximum)) => {
            // Upper-only bound (`max` declared, no `min` — e.g.
            // `nozzle_diameter` max=100, `PrintConfig.cpp:4835`):
            // exercise the declared ceiling and a seeded interior
            // anchored at 0 — a sampling anchor, not a schema
            // bound (an absent `min` is `-FLT_MAX`,
            // `Config.hpp:2475-2479`; e.g.
            // `wipe_tower_extra_rib_length` accepts negatives,
            // `PrintConfig.cpp:6941-6948`).
            let interior = seeded_interior(key, 0.0, maximum, option_type.contains("Int"));
            (
                vec![
                    numeric_case("max", maximum, entry, option_type),
                    numeric_case("seeded", interior, entry, option_type),
                ],
                omission,
            )
        }
        (Some(minimum), _) => {
            let interior = seeded_interior(
                key,
                minimum,
                minimum.max(1.0) * 4.0 + 1.0,
                option_type.contains("Int"),
            );
            let mut cases = vec![
                numeric_case("min", minimum, entry, option_type),
                numeric_case("seeded", interior, entry, option_type),
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
                let lit_max = literal_maximum(block).unwrap_or(minimum.max(1.0) * 4.0 + 1.0);
                extend_literals(
                    &mut cases,
                    entry,
                    [
                        ("min-literal", minimum),
                        (
                            "seeded-literal",
                            seeded_interior(key, minimum, lit_max, false),
                        ),
                    ],
                );
            }
            (cases, omission)
        }
        (None, None) => {
            // Integer ID/vector options retain baseline-only probes
            // because generic values can crash the original slicer.
            let unbounded_int = matches!(option_type, "coInt" | "coInts");
            let cases = if unbounded_int {
                vec![baseline_case()]
            } else if option_type == "coFloatOrPercent" {
                // Both percent and absolute-literal branches.
                vec![
                    case("probe-percent", "40%", entry),
                    numeric_case("probe-literal", 3.0, entry, "coFloat"),
                ]
            } else {
                let seed_hi = seeded_interior(key, 1.0, 9.0, option_type.contains("Int"));
                vec![
                    numeric_case("probe-min", 0.5, entry, option_type),
                    numeric_case("probe-seeded", seed_hi, entry, option_type),
                ]
            };
            (
                cases,
                if unbounded_int {
                    Some("baseline only; no injected numerical samples")
                } else {
                    omission
                },
            )
        }
    }
}

fn extend_literals(
    cases: &mut Vec<OptionCase>,
    entry: &Value,
    values: impl IntoIterator<Item = (&'static str, f64)>,
) {
    for (label, value) in values {
        let mut absolute = numeric_case(label, value, entry, "coFloat");
        absolute.label = label.to_owned();
        cases.push(absolute);
    }
}
