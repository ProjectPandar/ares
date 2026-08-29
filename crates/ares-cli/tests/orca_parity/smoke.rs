//! Printer smoke sweep across OrcaSlicer vendor machine presets.

use std::path::PathBuf;

use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, OrcaRunner},
};

fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

fn cube_model() -> PathBuf {
    runner::repo_root().join("tests/parity/cube10.stl")
}

/// Single-printer smoke case used to develop the harness and record the
/// first divergences on a plain Marlin i3 profile.
#[test]
fn orca_parity_ender3_smoke() {
    assert_printer_smoke("Creality", "Creality Ender-3 0.4 nozzle");
}

#[test]
fn orca_parity_artillery_x3_pro_smoke() {
    assert_printer_smoke("Artillery", "Artillery Sidewinder X3 Pro 0.4 nozzle");
}

#[test]
fn orca_parity_ratrig_vcast_smoke() {
    assert_printer_smoke("Ratrig", "RatRig V-Cast 0.4 nozzle");
}

fn assert_printer_smoke(vendor: &str, printer: &str) {
    let Some(runner) = OrcaRunner::from_env() else {
        eprintln!("skipping: no OrcaSlicer CLI available");
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), vendor).unwrap();
    let selection = parity::select_printer(&profiles, vendor, printer).unwrap();
    let case = parity::build_selection_case(&runner, &profiles, &selection, &cube_model()).unwrap();
    let outcome = parity::compare_case(&case);
    eprintln!(
        "printer smoke: {} {} {}",
        outcome.status, outcome.label, outcome.detail
    );
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

/// Full vendor sweep; writes `tests/parity/printer-smoke-summary.md` and
/// fails when any printer diverges, so each fix moves the summary to green.
/// Gated behind `ARES_PARITY_SWEEP=1` because slicing every vendor preset
/// through both slicers takes hours.
#[test]
fn orca_parity_printer_sweep() {
    if std::env::var("ARES_PARITY_SWEEP").as_deref() != Ok("1") {
        eprintln!("skipping: set ARES_PARITY_SWEEP=1 to run the full printer sweep");
        return;
    }
    let Some(runner) = OrcaRunner::from_env() else {
        eprintln!("skipping: no OrcaSlicer CLI available");
        return;
    };
    let root = profiles_root();
    let model = cube_model();
    let mut outcomes = Vec::new();
    for vendor in parity::vendors(&root) {
        let Ok(profiles) = VendorProfiles::load(&root, &vendor) else {
            continue;
        };
        for printer in profiles.instantiated_machine_names() {
            let selection = match parity::select_printer(&profiles, &vendor, &printer) {
                Ok(selection) => selection,
                Err(error) => {
                    let outcome = parity::ares_error(&format!("{vendor}/{printer}"), error);
                    eprintln!(
                        "[{}/sweep] {} {}",
                        outcomes.len() + 1,
                        outcome.status,
                        outcome.label
                    );
                    outcomes.push(outcome);
                    continue;
                }
            };
            let outcome = match parity::build_selection_case(&runner, &profiles, &selection, &model)
            {
                Ok(case) => parity::compare_case(&case),
                Err(error) => parity::ares_error(&format!("{vendor}/{printer}"), error),
            };
            eprintln!(
                "[{}/sweep] {} {}",
                outcomes.len() + 1,
                outcome.status,
                outcome.label
            );
            outcomes.push(outcome);
        }
    }
    write_summary(&outcomes);
    let failures = outcomes
        .iter()
        .filter(|outcome| outcome.status != "PASS")
        .count();
    assert!(
        failures == 0,
        "{failures}/{} printers diverge; see tests/parity/printer-smoke-summary.md",
        outcomes.len()
    );
}

fn write_summary(outcomes: &[parity::ParityOutcome]) {
    let passed = outcomes
        .iter()
        .filter(|outcome| outcome.status == "PASS")
        .count();
    let mut summary = format!(
        "# OrcaSlicer printer smoke summary\n\n{} of {} printers pass the semantic parity comparison (classic wall generator baseline; cube model).\n\n> NOTE: timing (M73/model-printing-time) is compared with `compare_ignoring_time` until the GCodeProcessor motion planner reaches Orca parity; timing deltas are therefore not reflected in the divergences below.\n\n| status | printer | first divergence |\n|---|---|---|\n",
        passed,
        outcomes.len()
    );
    for outcome in outcomes {
        summary.push_str(&format!(
            "| {} | {} | {} |\n",
            outcome.status,
            outcome.label,
            outcome.detail.replace('\n', " ")
        ));
    }
    let path = runner::repo_root().join("tests/parity/printer-smoke-summary.md");
    std::fs::write(&path, summary).expect("write printer smoke summary");
}
