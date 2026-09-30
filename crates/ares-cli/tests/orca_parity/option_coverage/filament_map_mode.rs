use super::*;
use crate::runner::CaseInputs;
use ares_core::ProjectFilamentMapMode;
use serde_json::json;

#[test]
fn filament_map_domain_contains_every_serializable_mode_not_the_ui_sentinel() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans
        .iter()
        .find(|plan| plan.key == "filament_map_mode")
        .unwrap();
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone())
            .collect::<Vec<_>>(),
        [
            Some(json!("Auto For Flush")),
            Some(json!("Auto For Match")),
            Some(json!("Manual"))
        ]
    );
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
    for case in &plan.cases {
        let raw = case.value.clone().unwrap();
        let typed: ProjectFilamentMapMode = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), raw);
    }
    assert!(serde_json::from_value::<ProjectFilamentMapMode>(json!("Default")).is_err());
}

#[test]
fn orca_parity_filament_map_wire_modes_match_full_output_and_ui_default_is_not_proof() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "filament_map_mode")
        .unwrap();
    assert_eq!(plan.cases.len(), 3);
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut cases = plan.cases.clone();
    cases.push(domains::OptionCase {
        label: "invalid-ui-default".to_owned(),
        value: Some(json!("Default")),
    });
    for case in &cases {
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
        assert_eq!(&process["filament_map_mode"], requested);
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("filament-map-wire/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let proof =
            runner::application::verify_value(&exported.project, "filament_map_mode", requested);
        if case.label == "invalid-ui-default" {
            assert_eq!(
                runner::application::settings(&exported.project).unwrap()["filament_map_mode"],
                "Auto For Flush"
            );
            assert!(
                proof.is_err(),
                "invalid UI token substitution must not count as application"
            );
            continue;
        }
        let proof = proof.unwrap();
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], *requested);
        if requested != "Auto For Flush" {
            assert!(runner::filament_map::verify(&exported.project, requested).is_err());
        }
        let exported = runner.bind_filament_map_mode(exported, requested).unwrap();
        let proof = runner::filament_map::verify(&exported.project, requested).unwrap();
        assert_eq!(proof["plate"]["effective"], *requested);
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference.lines().any(
                |line| line == format!("; filament_map_mode = {}", requested.as_str().unwrap())
            )
        );
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}
