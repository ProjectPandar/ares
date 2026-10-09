//! Bounded PrintConfig/Print::validate width candidates, not Flow success claims.
//! Locked widths additionally carry the finite actual-flow E2E scope below.
//!
//! `probes` are RECORDED-EXPECTATION entries (`execute:false`) documenting
//! the predicted oracle rejection for out-of-context / sentinel values;
//! they are not executed comparisons — the executed legal-context domain
//! (`cases`) is what the sweep verifies end-to-end.
use serde_json::{Value, json};

use super::{OptionCase, case, seeded_interior};

#[derive(Clone, Debug)]
pub(in crate::option_coverage) struct WidthDomain {
    pub(in crate::option_coverage) metadata: Value,
    pub(in crate::option_coverage) probes: Vec<Value>,
}

// Only the approved single-nozzle context is modeled. Mixed nozzles are deferred.
pub(in crate::option_coverage) fn generate(
    key: &str,
    entry: &Value,
    raw: (f64, f64),
    thick_bridges: bool,
    thick_internal_bridges: bool,
) -> (WidthDomain, Vec<OptionCase>) {
    let bridge = key == "bridge_line_width";
    let locked = matches!(key, "skin_infill_line_width" | "skeleton_infill_line_width");
    let lower = if bridge && thick_bridges && thick_internal_bridges {
        0.0
    } else {
        0.2
    };
    let upper = if bridge { 0.4 } else { 2.0 };
    let mut cases = Vec::new();
    let mut units = Vec::new();
    let mut probes = Vec::new();
    for (unit, suffix, scale) in [("mm", "", 1.0), ("percent", "%", 250.0)] {
        let (minimum, maximum) = (lower * scale, upper * scale);
        let seed = seeded_interior(&format!("{key}/{unit}"), minimum, maximum, false);
        for (label, value) in [("zero", 0.0), ("context-max", maximum), ("interior", seed)] {
            let rendered = render(value, suffix);
            if label == "zero" && !suffix.is_empty() && !bridge {
                // A percent-form zero has no mm-zero "auto" fallback: the
                // oracle CLI dies on the resolved-zero width (exit 156),
                // so the sentinel is a non-executed probe.
                probes.push(json!({"wire": rendered,
                    "expected": "oracle-process-failure", "validation_sentinel": true,
                    "execute": false}));
                continue;
            }
            cases.push(case(&format!("{unit}-{label}"), &rendered, entry));
        }
        if locked {
            let near_lower = if suffix.is_empty() {
                "0.20001"
            } else {
                "50.001%"
            };
            cases.push(case(&format!("{unit}-near-lower"), near_lower, entry));
        }
        units.push(json!({
            "unit": unit, "raw_schema": {"min": raw.0, "max": raw.1,
                "endpoint_semantics": "ConfigOptionDef::is_value_valid precision=4 approximate endpoints; min=0 rejects negatives"},
            "positive_context": {"lower": minimum, "lower_open": true, "upper": maximum, "upper_open": false},
            "zero": {"wire": render(0.0, suffix), "validation_sentinel": true,
                "flow": if bridge { "solid-infill fallback" } else if suffix.is_empty() { "auto" } else if locked { "zero resolved width; Flow-spacing rejection verified by separate E2E" } else { "zero resolved width; runtime unverified" }}
        }));
        // Adding one does not advance a FLT_MAX ceiling in f64 arithmetic.
        for value in [-1.0, raw.1 + (raw.1 * 0.001).max(1.0)] {
            probes.push(json!({"wire": render(value, suffix), "expected": "raw-schema-rejection", "execute": false}));
        }
        if raw.1 > maximum {
            probes.push(json!({"wire": render(raw.1, suffix), "expected": "context-rejection", "raw_schema_endpoint": true, "execute": false}));
        }
        if minimum > 0.0 {
            probes.push(json!({"wire": render(minimum, suffix), "expected": "context-rejection", "execute": false}));
        }
    }
    let mut dependencies = json!({"nozzle_diameter": ["0.4"], "layer_height": "0.2",
        "thick_bridges": if thick_bridges { "1" } else { "0" },
        "thick_internal_bridges": if thick_internal_bridges { "1" } else { "0" }});
    if locked {
        dependencies["sparse_infill_pattern"] = json!("lockedzag");
        dependencies["sparse_infill_density"] = json!("40%");
    }
    let mut metadata = json!({
        "scope": "single-nozzle validation candidates; not actual-default or slicing coverage",
        "gui_missing_percent_threshold": 10, "units": units,
        "dependencies": dependencies,
        "export_required": ["nozzle_diameter", "layer_height", "thick_bridges", "thick_internal_bridges", "internal_solid_infill_line_width"],
        "deferred": ["mixed nozzles", "object/region overrides", "Flow/slicing feasibility", "tolerance edges"]
    });
    if locked {
        metadata["scope"] = json!(
            "bounded LockedZag 40% admission and actual sparse flow; one 0.4mm nozzle, 0.2mm layers"
        );
        metadata["dependency_control"] = json!({
            "nozzle_diameter": ["0.6"], "layer_height": "0.3",
            "test": "orca_parity_locked_width_upper_bound_tracks_single_nozzle_diameter"
        });
        metadata["deferred"] = json!([
            "mixed nozzles",
            "object/region overrides",
            "bridge LockedZag flow reuse",
            "adaptive/painted layer heights",
            "arbitrary Flow feasibility",
            "tolerance edges"
        ]);
    }
    (WidthDomain { metadata, probes }, cases)
}

fn render(value: f64, suffix: &str) -> String {
    // The 3mf export serialises float-or-percent values with limited
    // precision — five decimals for millimetres, three for percent — so
    // requesting more digits cannot round-trip through the export
    // (`option/line_width/mm-interior` 1.134222 -> 1.13422,
    // `percent-interior` 366.7208% -> 366.721%).
    let text = if suffix == "%" {
        format!("{value:.3}")
    } else {
        format!("{value:.5}")
    };
    format!(
        "{}{suffix}",
        text.trim_end_matches('0').trim_end_matches('.')
    )
}
