//! AdaptivePAProcessor.cpp:214-284: active bridge override dependency boundaries.

use super::{OptionOutcome, domains};
use crate::VendorProfiles;
use crate::{CaseInputs, OrcaRunner, runner};
use serde_json::json;
use std::fs;

#[test]
fn orca_parity_adaptive_pa_bridge_bounded_option_classification() {
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
        ("retained-interior", "0.5", "1", "1", true),
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
            ("gcode_comments", "1"),
            ("enable_overhang_speed", "1"),
            ("enable_prime_tower", "0"),
            ("seam_slope_type", "none"),
        ];
        for (key, value) in process_values {
            process.insert(key.into(), json!(value));
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
                    label: &format!("option/adaptive-pa-bridges/context/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        for (key, value) in process_values {
            runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
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
        let outcome = if label == "interior" {
            crate::compare_exported(&runner, &exported)
        } else {
            crate::compare_case(&sliced)
        };
        fs::write(
            case_dir.join("result.json"),
            serde_json::to_vec_pretty(&json!({
                "label": label, "bridge": bridge, "pa": pa, "adaptive": adaptive,
                "valid_model": valid, "pa_tags": tags, "status": outcome.status,
                "detail": outcome.detail,
                "scope": "six finite requests/single-slot dependency controls; not global schema/vector coverage"
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
    let plans = domains::load(&runner::repo_root());
    let plan = plans
        .iter()
        .find(|plan| plan.key == "adaptive_pressure_advance_bridges")
        .unwrap();
    let outcome = OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None);
    assert_eq!(
        outcome.status, "PASS",
        "Native full comparisons passed, but bounded option classification remains {}: {}",
        outcome.status, outcome.detail
    );
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| (case.label.as_str(), case.value.clone()))
            .collect::<Vec<_>>(),
        [
            ("disabled-override", Some(json!(["0"]))),
            ("threshold-equal", Some(json!(["0.0001"]))),
            ("threshold-above", Some(json!(["0.000101"]))),
            ("interior", Some(json!(["0.06"]))),
            ("probe-min", Some(json!(["0.5"]))),
            ("probe-max", Some(json!(["2.0"]))),
        ]
    );
    let mut unproven = plan.clone();
    unproven.provably_legal = false;
    let limited = OptionOutcome::executed(&unproven, unproven.cases.len(), Vec::new(), 0, None);
    assert_eq!(
        limited.status, "INCOMPLETE",
        "matching samples cannot prove an undeclared domain"
    );
}

type PresetSettings = (
    serde_json::Map<String, serde_json::Value>,
    serde_json::Map<String, serde_json::Value>,
    Vec<serde_json::Map<String, serde_json::Value>>,
);

fn settings() -> PresetSettings {
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), json!("0"));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    (machine, process, filaments)
}

pub(super) const KEY: &str = "adaptive_pressure_advance_bridges";
const MODEL: &str = "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000";

pub(super) fn cases() -> Vec<domains::OptionCase> {
    [
        ("disabled-override", "0"),
        ("threshold-equal", "0.0001"),
        ("threshold-above", "0.000101"),
        ("interior", "0.06"),
        ("probe-min", "0.5"),
        ("probe-max", "2.0"),
    ]
    .into_iter()
    .map(|(label, value)| domains::OptionCase {
        label: label.into(),
        value: Some(json!([value])),
    })
    .collect()
}

fn dependencies() -> serde_json::Map<String, serde_json::Value> {
    serde_json::Map::from_iter([
        ("gcode_comments".into(), json!("1")),
        ("enable_overhang_speed".into(), json!("1")),
        ("enable_prime_tower".into(), json!("0")),
        ("seam_slope_type".into(), json!("none")),
        ("enable_pressure_advance".into(), json!(["1"])),
        ("pressure_advance".into(), json!(["0.035"])),
        ("adaptive_pressure_advance".into(), json!(["1"])),
        ("adaptive_pressure_advance_model".into(), json!([MODEL])),
        ("adaptive_pressure_advance_overhangs".into(), json!(["0"])),
    ])
}

pub(super) fn metadata() -> serde_json::Value {
    json!({
        "scope":"six finite requests in one logical-filament/non-calibration/no-tower bridge context; not a global real/vector domain",
        "source":"PrintConfig.cpp:2401-2408; GCode.cpp:6720-6745,6816-6841; AdaptivePAProcessor.cpp:214-295",
        "raw_minimum":"inherited -FLT_MAX, not covered", "explicit_maximum":2,
        "disable_sentinel":0, "effective_threshold":0.0001,
        "geometry":"tests/parity/overhang.stl", "dependencies":dependencies(),
        "deferred":["negative/extreme/schema rejection", "nonfinite/malformed", "multiple logical slots/short vectors", "calibration", "lookahead/reset", "sloped mixed roles/overhang ramps", "hardware/other firmware"],
    })
}

pub(super) fn activate(
    process: &mut serde_json::Map<String, serde_json::Value>,
    filaments: &mut [serde_json::Map<String, serde_json::Value>],
) {
    for (key, value) in dependencies() {
        if value.is_array() {
            filaments[0].insert(key, value);
        } else {
            process.insert(key, value);
        }
    }
}

pub(super) fn verify(
    project: &[u8],
    requested: &serde_json::Value,
) -> Result<serde_json::Value, runner::stages::StageError> {
    let mut expected = dependencies();
    expected.insert(KEY.into(), requested.clone());
    let mut proofs = serde_json::Map::new();
    for (key, value) in expected {
        proofs.insert(
            key.clone(),
            runner::application::verify_value(project, &key, &value)?,
        );
    }
    Ok(json!({"domain":metadata(),"requested":requested,"application":proofs}))
}

pub(super) fn compare(
    runner: &OrcaRunner,
    exported: &runner::ExportedCase,
    requested: &serde_json::Value,
) -> crate::ParityOutcome {
    let sliced = runner.slice_case(exported).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(native.contains(";TYPE:Bridge\n"), "Native bridge absent");
    assert!(!native.contains(";TYPE:Prime tower\n"));
    let tags = native
        .lines()
        .filter(|line| line.starts_with("; PA_CHANGE:"))
        .collect::<Vec<_>>();
    assert!(tags.iter().any(|line| line.contains(" BR:1 ")));
    assert!(tags.iter().any(|line| line.contains(" BR:0 ")));
    assert_eq!(
        native.contains("; APA Model Override (bridge)\n"),
        requested[0].as_str().unwrap().parse::<f64>().unwrap() > 1e-4
    );
    // Preserve the Option sweep's existing full-stream Native ND path.
    crate::compare_exported_for(runner, exported, Some(KEY))
}
