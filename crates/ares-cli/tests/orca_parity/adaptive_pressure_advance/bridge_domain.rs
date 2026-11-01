//! AdaptivePAProcessor.cpp:214-284: active bridge override dependency boundaries.

use super::settings;
use crate::{CaseInputs, OrcaRunner, runner};
use serde_json::json;
use std::fs;

#[test]
fn adaptive_pa_bridge_effective_domain_and_dependencies() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("adaptive-pa-bridges-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let mut failures = Vec::new();
    for (label, bridge, pa, adaptive, valid) in [
        ("disabled-override", "0", "1", "1", true),
        ("threshold-equal", "0.0001", "1", "1", true),
        // Config.hpp:846-865,910-912 uses default stream precision;
        // libslic3r.h:52 defines EPSILON=1e-4. This survives export above it.
        ("threshold-above", "0.000101", "1", "1", true),
        ("interior", "0.06", "1", "1", true),
        ("upper", "2", "1", "1", true),
        ("disabled-pa", "0.06", "0", "1", true),
        ("disabled-adaptive", "0.06", "1", "0", true),
        ("invalid-model", "0.06", "1", "1", false),
    ] {
        let (machine, mut process, mut filaments) = settings();
        let model = if valid {
            "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000"
        } else {
            "not-a-model"
        };
        let filament_values = [
            ("enable_pressure_advance", pa),
            ("pressure_advance", "0.035"),
            ("adaptive_pressure_advance", adaptive),
            ("adaptive_pressure_advance_model", model),
            ("adaptive_pressure_advance_overhangs", "0"),
            ("adaptive_pressure_advance_bridges", bridge),
        ];
        let process_values = [
            ("gcode_comments", json!("1")),
            ("enable_overhang_speed", json!(["1"])),
            ("enable_prime_tower", json!("0")),
            ("seam_slope_type", json!("none")),
        ];
        for (key, value) in &process_values {
            process.insert((*key).into(), value.clone());
        }
        for filament in &mut filaments {
            for (key, value) in filament_values {
                filament.insert(key.into(), json!([value]));
            }
        }
        let case_dir = directory.join(label);
        fs::create_dir(&case_dir).unwrap();
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("adaptive-pa/bridge-domain/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        for (key, value) in process_values {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        for (key, value) in filament_values {
            runner::application::verify_value(&exported.project, key, &json!([value])).unwrap();
        }
        fs::write(case_dir.join("input.3mf"), &exported.project).unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        fs::write(case_dir.join("native.gcode"), &sliced.reference).unwrap();
        let native = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            native.contains(";TYPE:Bridge\n"),
            "Native bridge absent: {label}"
        );
        assert!(!native.contains(";TYPE:Prime tower\n"));
        let active = pa == "1" && adaptive == "1";
        let tags: Vec<_> = native
            .lines()
            .filter(|line| line.starts_with("; PA_CHANGE:"))
            .collect();
        if active {
            assert!(tags.iter().any(|line| line.contains(" BR:1 ")));
            assert!(tags.iter().any(|line| line.contains(" BR:0 ")));
            // The source's debug-comment predicate is outside the valid-model
            // branch, so invalid models can print this comment with static PA.
            assert_eq!(
                native.contains("; APA Model Override (bridge)\n"),
                bridge.parse::<f64>().unwrap() > 1e-4
            );
        } else {
            assert!(tags.is_empty());
        }
        if !valid {
            assert!(
                native
                    .contains("; APA: Interpolator setup failed, using default pressure advance\n")
            );
            assert!(
                native
                    .lines()
                    .filter(|line| line.starts_with("M900 K"))
                    .all(|line| line == "M900 K0.035; Override pressure advance value")
            );
        }
        let outcome = crate::compare_case(&sliced);
        fs::write(
            case_dir.join("result.json"),
            serde_json::to_vec_pretty(&json!({
                "label": label, "bridge": bridge, "pa": pa, "adaptive": adaptive,
                "valid_model": valid, "pa_tags": tags, "status": outcome.status,
                "detail": outcome.detail,
                "scope": "contextual effective domain; not global schema/vector coverage"
            }))
            .unwrap(),
        )
        .unwrap();
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
    }
    eprintln!("Adaptive PA bridge artifacts: {}", directory.display());
    assert!(
        failures.is_empty(),
        "{}; {directory:?}",
        failures.join("\n")
    );
}
