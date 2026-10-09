//! PrintConfig.cpp:2215-2248,10436-10447; Print.cpp:613-930,1284-1306.
//! Bounded G2 witnesses; no global physical upper-bound claim.

use super::super::{OptionOutcome, domains, inject_case, profiles_root};
use crate::{
    presets::VendorProfiles,
    runner::{self, CaseInputs, OrcaRunner},
};
use serde_json::json;

#[path = "clearance/geometry.rs"]
mod geometry;

#[test]
fn orca_parity_g2_lid_loaded_and_complete_output() {
    loaded_and_complete(
        "extruder_clearance_height_to_lid",
        &["0.00001", "0.5", "250"],
    );
}

#[test]
fn orca_parity_g2_rod_loaded_and_complete_output() {
    loaded_and_complete(
        "extruder_clearance_height_to_rod",
        &["0.00001", "0.5", "250"],
    );
}

#[test]
fn orca_parity_g2_radius_loaded_and_complete_output() {
    loaded_and_complete("extruder_clearance_radius", &["0.00001", "0.5", "250"]);
}

#[test]
fn orca_parity_g2_nozzle_loaded_and_complete_output() {
    loaded_and_complete("nozzle_height", &["0.00001", "1", "400"]);
}

fn loaded_and_complete(key: &str, values: &[&str]) {
    if crate::native_environment::binary("ARES_ORCA_NATIVE_BIN").is_none()
        || crate::native_environment::value("ARES_ORCA_RESOURCES_DIR").is_none()
    {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|p| p.key == key)
        .unwrap();
    let mut proofs = Vec::new();
    for wire in values {
        let case = domains::OptionCase {
            label: format!("loaded-{wire}"),
            value: Some(json!(wire)),
        };
        let exported = export(&runner, &plan, &case).unwrap();
        let effective = runner::application::settings(&exported.project).unwrap();
        // Float serialization may use scientific notation; equality is exact, not a tolerance.
        assert_eq!(
            effective[key].as_str().unwrap().parse::<f64>().unwrap(),
            wire.parse::<f64>().unwrap()
        );
        runner::application::verify_value(&exported.project, "print_sequence", &json!("by layer"))
            .unwrap();
        runner::application::verify_value(&exported.project, "nozzle_diameter", &json!(["0.4"]))
            .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = crate::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{key}={wire}: {}", outcome.detail);
        proofs.push(json!({"requested":wire,"effective":effective[key],"status":outcome.status,"artifacts":outcome.artifacts}));
    }
    for wire in ["0", "-0.00001"] {
        let case = domains::OptionCase {
            label: format!("rejected-{wire}"),
            value: Some(json!(wire)),
        };
        let error = export(&runner, &plan, &case)
            .err()
            .expect("nonpositive clearance must reject");
        assert_eq!(error.stage, runner::stages::Stage::Export);
        assert_eq!(error.kind, runner::stages::FailureKind::Process);
        let captured = error.captured.unwrap();
        let code = if cfg!(unix) { 238 } else { -18 };
        assert_eq!(captured.code, Some(code));
        let options: ares_core::SliceOptions = serde_json::from_value(json!({key:wire})).unwrap();
        let errors = options.validate_extruder_clearance_options().unwrap();
        assert_eq!(errors.len(), 1);
        assert!(
            String::from_utf8_lossy(&captured.stderr).contains(&format!("{key}: {}", errors[key]))
        );
        proofs.push(json!({"rejected":wire,"stage":"Export","code":captured.code,"evidence":error.evidence,"validation":errors}));
    }
    let mut observed_plan = plan.clone();
    observed_plan.cases = values
        .iter()
        .map(|wire| domains::OptionCase {
            label: format!("loaded-{wire}"),
            value: Some(json!(wire)),
        })
        .collect();
    let outcome = OptionOutcome::executed(&observed_plan, values.len(), Vec::new(), 0, None);
    let root = crate::artifacts::root_from_env().unwrap();
    crate::artifacts::overwrite(&root.join(format!("g2-{key}-loaded.json")), &serde_json::to_vec_pretty(&json!({
        "scope":"one object, by layer, 0.4mm nozzle; sequential consumers require separate witnesses",
        "proofs":proofs,"classification":outcome.status,"detail":outcome.detail
    })).unwrap()).unwrap();
    assert_eq!(outcome.status, "PASS", "{key}: {}", outcome.detail);
    let authored = plan
        .cases
        .iter()
        .map(|case| case.value.as_ref().unwrap().as_str().unwrap())
        .collect::<Vec<_>>();
    for wire in values {
        assert!(authored.contains(wire), "{key}: missing canonical {wire}");
    }
}

fn export(
    runner: &OrcaRunner,
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    inject_case(plan, case, &mut machine, &mut process, &mut filaments);
    runner.export_case(
        &CaseInputs {
            label: &format!("g2/{}/{}", plan.key, case.label),
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &crate::smoke_overrides(),
        &runner::repo_root().join("tests/parity/cube10.stl"),
    )
}
