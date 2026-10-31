//! Public BBL timelapse / first-travel writer-state regressions.

use super::{OrcaRunner, VendorProfiles};
use crate::runner::{CaseInputs, application, repo_root};
use serde_json::{Map, json};

#[test]
fn stock_bbl_timelapse_z_restatement() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let mut failures = Vec::new();
    for model in ["Bambu Lab H2S", "Bambu Lab P1P", "Bambu Lab P1S"] {
        for nozzle in ["0.2", "0.4", "0.6", "0.8"] {
            let outcome = compare(&runner, &profiles, model, nozzle, false);
            if outcome.status != "PASS" {
                failures.push(format!("{model}/{nozzle}: {}", outcome.detail));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn stock_bbl_timelapse_z_comments() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let outcome = compare(&runner, &profiles, "Bambu Lab P1P", "0.2", true);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn compare(
    runner: &OrcaRunner,
    profiles: &VendorProfiles,
    model: &str,
    nozzle: &str,
    comments: bool,
) -> crate::ParityOutcome {
    let printer = format!("{model} {nozzle} nozzle");
    let selection = crate::select_printer(profiles, "BBL", &printer).unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    if comments {
        process.insert("gcode_comments".into(), json!("1"));
    }
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("bbl-timelapse/{printer}/comments-{comments}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &Map::new(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    application::verify_value(&exported.project, "nozzle_diameter", &json!([nozzle])).unwrap();
    if comments {
        application::verify_value(&exported.project, "gcode_comments", &json!("1")).unwrap();
    }
    crate::compare_exported(runner, &exported)
}
