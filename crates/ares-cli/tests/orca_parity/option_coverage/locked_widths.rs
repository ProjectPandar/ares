use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn locked_width_plans_cover_context_edges_and_seeded_interiors() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in ["skin_infill_line_width", "skeleton_infill_line_width"] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let wires = plan
            .cases
            .iter()
            .map(|case| case.value.as_ref().unwrap().as_str().unwrap())
            .collect::<Vec<_>>();
        for expected in ["0", "0.20001", "2", "50.001%", "500%"] {
            assert!(
                wires.contains(&expected),
                "{key}: missing context edge {expected}"
            );
        }
        for (percent, lower, upper) in [(false, 0.3, 1.99), (true, 75.0, 499.0)] {
            assert!(
                plan.cases.iter().any(|case| {
                    let width = runner::application::width(case.value.as_ref().unwrap()).unwrap();
                    width.percent == percent && width.value > lower && width.value < upper
                }),
                "{key}: missing seeded interior, percent={percent}"
            );
        }
        assert!(
            !plan.provably_legal,
            "one context cannot close the global range"
        );
    }
}

#[test]
fn orca_parity_locked_skin_width_drives_sparse_extrusion() {
    compare_width("skin_infill_line_width");
}

#[test]
fn orca_parity_locked_skeleton_width_drives_sparse_extrusion() {
    compare_width("skeleton_infill_line_width");
}

fn compare_width(key: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == key)
        .unwrap();
    let mut prior_extrusions: Option<(f64, Vec<String>)> = None;
    let mut proofs = Vec::new();
    for case in &plan.cases {
        let value = case.value.as_ref().unwrap();
        let wire = value.as_str().unwrap();
        let requested = runner::application::width(value).unwrap();
        let width = if requested.percent {
            requested.value * 0.004
        } else if requested.value == 0.0 {
            0.45
        } else {
            requested.value
        };
        let exported = export_width(&runner, &plan, case).unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let mut sparse = false;
        let mut current_width = 0.0_f64;
        let mut extrusions = Vec::new();
        for line in std::str::from_utf8(&sliced.reference).unwrap().lines() {
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
                extrusions.push(line.to_owned());
            }
        }
        assert!(
            !extrusions.is_empty(),
            "{key}={wire} must drive sparse extrusion"
        );
        if let Some((prior_width, prior)) = prior_extrusions
            && (prior_width - width).abs() > 1e-6
        {
            assert_ne!(
                prior, extrusions,
                "width probes must change actual extrusion"
            );
        }
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{key}={wire}: {}", outcome.detail);
        proofs.push(json!({"label": case.label, "requested": value,
            "effective_width_mm": width, "sparse_extrusions": extrusions.len(),
            "full_output_status": outcome.status}));
        prior_extrusions = Some((width, extrusions));
    }
    let outcome = OptionOutcome::executed(&plan, proofs.len(), Vec::new(), 0, None);
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join(format!("{key}-bounded.json")),
        &serde_json::to_vec_pretty(&json!({
            "scope": "LockedZag 40%, one 0.4mm nozzle, 0.2mm layers; not global width coverage",
            "domain": plan.width_domain.as_ref().unwrap().metadata,
            "cases": proofs, "classification": outcome.status, "detail": outcome.detail,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{key}: {}", outcome.detail);
    assert!(plan.omission.is_none(), "{key}");
}

#[test]
fn original_orca_rejects_locked_width_context_boundaries() {
    use runner::stages::{FailureKind, Stage};
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plans = domains::load(&runner::repo_root());
    for key in ["skin_infill_line_width", "skeleton_infill_line_width"] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        for (wire, stage, code, diagnostic) in [
            ("0.2", Stage::Slice, -51_i32, "Too small line width"),
            ("50%", Stage::Slice, -51, "Too small line width"),
            ("2.01", Stage::Export, -18, "too large line width"),
            ("501%", Stage::Export, -18, "too large line width"),
            (
                "0%",
                Stage::Slice,
                -100,
                "Flow::spacing() produced negative spacing",
            ),
        ] {
            let case = domains::OptionCase {
                label: format!("rejected-{wire}"),
                value: Some(json!(wire)),
            };
            let error = export_width(&runner, plan, &case)
                .and_then(|exported| runner.slice_case(&exported))
                .err()
                .expect("invalid width must fail");
            assert_eq!(error.stage, stage);
            assert_eq!(error.kind, FailureKind::Process);
            if stage == Stage::Export {
                assert!(
                    error.input_rejection(key),
                    "upper-bound rejection must name {key}"
                );
            }
            assert!(
                error
                    .evidence
                    .as_ref()
                    .unwrap()
                    .join("status.json")
                    .is_file()
            );
            let captured = error.captured.unwrap();
            let expected_code = if cfg!(unix) {
                code.rem_euclid(256)
            } else {
                code
            };
            assert_eq!(captured.code, Some(expected_code), "{key}={wire}");
            let diagnostic_text = format!(
                "{}\n{}",
                String::from_utf8_lossy(&captured.stdout),
                String::from_utf8_lossy(&captured.stderr)
            );
            assert!(
                diagnostic_text.contains(diagnostic),
                "{key}={wire}: {diagnostic_text}"
            );
        }
    }
}

fn export_width(
    runner: &OrcaRunner,
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    export_width_context(runner, plan, case, "0.4", "0.2")
}

pub(super) fn export_width_context(
    runner: &OrcaRunner,
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
    nozzle: &str,
    height: &str,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    inject_case(plan, case, &mut machine, &mut process, &mut filaments);
    machine.insert("nozzle_diameter".into(), json!([nozzle]));
    process.insert("layer_height".into(), json!(height));
    let exported = runner.export_case(
        &CaseInputs {
            label: &format!(
                "locked-width/{nozzle}/{}/{}/{}",
                height, plan.key, case.label
            ),
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &parity::smoke_overrides(),
        &runner::repo_root().join("tests/parity/cube10.stl"),
    )?;
    runner::application::verify_value(&exported.project, &plan.key, case.value.as_ref().unwrap())
        .unwrap();
    for (option, value) in [
        ("sparse_infill_pattern", json!("lockedzag")),
        ("sparse_infill_density", json!("40%")),
        ("use_relative_e_distances", json!("1")),
        ("nozzle_diameter", json!([nozzle])),
        ("layer_height", json!(height)),
    ] {
        runner::application::verify_value(&exported.project, option, &value).unwrap();
    }
    Ok(exported)
}
