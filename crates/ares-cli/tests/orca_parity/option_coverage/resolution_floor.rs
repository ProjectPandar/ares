use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn resolution_samples_do_not_claim_the_unprobed_implicit_ceiling() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans.iter().find(|plan| plan.key == "resolution").unwrap();
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
    assert!(plan.cases.iter().all(|case| case.label != "max"));
    assert!(
        plan.cases
            .iter()
            .any(|case| case.label == "min" && case.value == Some(json!("0")))
    );
    assert!(plan.cases.iter().any(|case| case.label == "seeded"));
}

#[test]
fn orca_parity_resolution_floor_boundaries_match_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "resolution")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut cases = plan.cases.clone();
    cases.extend(
        [
            ("below", "0.0005"),
            ("just-below", "0.000999"),
            ("threshold", "0.001"),
            ("above", "0.001001"),
        ]
        .map(|(label, value)| domains::OptionCase {
            label: label.to_owned(),
            value: Some(json!(value)),
        }),
    );
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
        assert_eq!(&process["resolution"], requested);
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("resolution-floor/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let expected = match case.label.as_str() {
            "min" | "below" | "just-below" | "threshold" => json!("0.001"),
            "above" => json!("0.001001"),
            "seeded" => requested.clone(),
            other => panic!("unexpected resolution case {other}"),
        };
        let proof =
            runner::application::verify_value(&exported.project, "resolution", requested).unwrap();
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], expected);
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference
                .lines()
                .any(|line| line == format!("; resolution = {}", expected.as_str().unwrap()))
        );
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}
