//! Print.cpp:1543-1556 / Fill.cpp:990-1000: a second single-nozzle control.
//! Complete output comparison is required; this does not cover mixed nozzles.

use super::*;
use serde_json::json;

#[test]
fn orca_parity_locked_width_upper_bound_tracks_single_nozzle_diameter() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plans = domains::load(&runner::repo_root());
    let mut proofs = Vec::new();
    for key in ["skin_infill_line_width", "skeleton_infill_line_width"] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        for (label, wire, width) in [
            ("mm-auto", "0", 0.675_f64),
            ("mm-near-lower", "0.30001", 0.30001),
            ("mm-upper", "3", 3.0),
            ("percent-near-lower", "50.001%", 0.300006),
            ("percent-upper", "500%", 3.0),
        ] {
            let case = domains::OptionCase {
                label: label.into(),
                value: Some(json!(wire)),
            };
            let exported =
                locked_widths::export_width_context(&runner, plan, &case, "0.6", "0.3").unwrap();
            let sliced = runner.slice_case(&exported).unwrap();
            let extrusions = sparse_extrusions(&sliced.reference, width);
            assert!(extrusions > 0, "{key}={wire} must drive sparse extrusion");
            let outcome = parity::compare_case(&sliced);
            assert_eq!(outcome.status, "PASS", "{key}={wire}: {}", outcome.detail);
            proofs.push(json!({"key": key, "requested": wire,
                "effective_width_mm": width, "sparse_extrusions": extrusions,
                "full_output_status": outcome.status}));
        }
    }
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("locked-width-second-context.json"),
        &serde_json::to_vec_pretty(&json!({
            "nozzle_diameter": ["0.6"], "layer_height": "0.3",
            "scope": "second single-nozzle dependency control; not global width coverage",
            "cases": proofs,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn sparse_extrusions(reference: &[u8], width: f64) -> usize {
    let mut sparse = false;
    let mut current_width = 0.0_f64;
    let mut extrusions = 0;
    for line in std::str::from_utf8(reference).unwrap().lines() {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            sparse = role == "Sparse infill";
        } else if let Some(value) = line.strip_prefix(";WIDTH:") {
            current_width = value.parse().unwrap();
        } else if sparse
            && (current_width - width).abs() < 1e-6
            && (line.starts_with("G1 X") || line.starts_with("G1 Y"))
            && line.split_whitespace().any(|word| {
                word.strip_prefix('E')
                    .and_then(|value| value.parse::<f64>().ok())
                    .is_some_and(|value| value > 0.0)
            })
        {
            extrusions += 1;
        }
    }
    extrusions
}
