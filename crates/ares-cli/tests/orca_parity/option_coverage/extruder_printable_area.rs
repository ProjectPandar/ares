//! PrintConfig.cpp:692-696 / Config.hpp:1605-1717: bounded grouped-point
//! admission on Ender-3, not all polygon or multi-nozzle geometry coverage.

use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_extruder_printable_area_grouped_requests_complete_bounded_scope() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let mut plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "extruder_printable_area")
        .unwrap();
    let authored = plan.cases.clone();
    let full = "0x0,220x0,220x220,0x220";
    let inset = "10x10,210x10,210x210,10x210";
    // ConfigOptionPointsGroups::vserialize: one string per polygon.
    let requests = [
        ("probe-bed", json!([full])),
        ("probe-inset", json!([inset])),
        ("probe-overlap", json!([full, inset])),
    ];
    plan.cases = requests
        .iter()
        .map(|(label, value)| domains::OptionCase {
            label: (*label).into(),
            value: Some(value.clone()),
        })
        .collect();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut proofs = Vec::new();
    for case in &plan.cases {
        let mut machine = profiles.machine(&selection.printer).unwrap();
        let mut process = profiles.process(&selection.process).unwrap();
        parity::normalize_process_defaults(&machine, &mut process);
        let mut filaments = selection
            .filaments
            .iter()
            .map(|name| profiles.filament(name).unwrap())
            .collect::<Vec<_>>();
        parity::normalize_filament_defaults(&mut filaments);
        inject_case(&plan, case, &mut machine, &mut process, &mut filaments);
        let requested = case.value.as_ref().unwrap();
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("extruder-printable-area/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let proof =
            runner::application::verify_value(&exported.project, &plan.key, requested).unwrap();
        let settings = runner::application::settings(&exported.project).unwrap();
        assert_eq!(settings[&plan.key], *requested, "{}", case.label);
        assert_eq!(
            settings[&plan.key].as_array().unwrap().len(),
            requested.as_array().unwrap().len(),
            "{}",
            case.label
        );
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
        proofs.push(json!({
            "label": case.label, "application": proof,
            "requested_groups": requested.as_array().unwrap().len(),
            "full_output_status": outcome.status,
        }));
    }
    let outcome = OptionOutcome::executed(&plan, proofs.len(), Vec::new(), 0, None);
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("extruder-printable-area-bounded.json"),
        &serde_json::to_vec_pretty(&json!({
            "scope": "three grouped-point admission cases; not all polygons or multi-nozzle geometry",
            "cases": proofs, "classification": outcome.status, "detail": outcome.detail,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert!(plan.omission.is_none());
    assert_eq!(authored.len(), requests.len());
    for (case, (label, requested)) in authored.iter().zip(&requests) {
        assert_eq!(case.label, *label);
        assert_eq!(case.value.as_ref(), Some(requested));
    }
}
