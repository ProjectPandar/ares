use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn cut_retraction_plan_covers_all_named_modes_without_claiming_integer_completion() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans
        .iter()
        .find(|plan| plan.key == "enable_long_retraction_when_cut")
        .unwrap();
    let values = plan
        .cases
        .iter()
        .map(|case| case.value.as_ref().unwrap().as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(values, ["0", "1", "2"]);
    assert!(!plan.provably_legal);
    let outcome = OptionOutcome::executed(plan, values.len(), vec![], 0, None);
    assert_eq!(outcome.status, "INCOMPLETE");
}

#[test]
fn orca_parity_cut_retraction_modes_select_runtime_overrides() {
    compare_modes(false);
}

#[test]
fn orca_parity_cut_retraction_modes_preserve_false_filament_override() {
    compare_modes(true);
}

fn compare_modes(machine_enabled: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "enable_long_retraction_when_cut")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
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
        if machine_enabled {
            machine.insert("long_retractions_when_cut".into(), json!(["1"]));
            filaments[0].insert("filament_long_retractions_when_cut".into(), json!(["0"]));
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("cut-retraction/machine-{machine_enabled}/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        runner::application::verify_value(
            &exported.project,
            &plan.key,
            case.value.as_ref().unwrap(),
        )
        .unwrap();
        for (key, value) in [
            (
                "long_retractions_when_cut",
                json!([if machine_enabled { "1" } else { "0" }]),
            ),
            ("retraction_distances_when_cut", json!(["12"])),
            (
                "filament_long_retractions_when_cut",
                json!([if machine_enabled { "0" } else { "1" }]),
            ),
            ("filament_retraction_distances_when_cut", json!(["16"])),
            ("use_relative_e_distances", json!("1")),
        ] {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let witness = reference
            .split_once(";CUT_GATE_BEGIN\n")
            .unwrap()
            .1
            .split_once(";CUT_GATE_END")
            .unwrap()
            .0;
        let (expected, expected_moves): (&str, &[&str]) = match (
            case.value.as_ref().unwrap().as_str().unwrap(),
            machine_enabled,
        ) {
            ("0" | "1", false) => (";CUT_GATE false 12\n", &[]),
            ("2", false) => (";CUT_GATE true 16\nG1 E-16 F1800\n", &["G1 E-16 F1800"]),
            ("0" | "1", true) => (";CUT_GATE true 12\nG1 E-12 F1800\n", &["G1 E-12 F1800"]),
            ("2", true) => (";CUT_GATE false 16\n", &[]),
            (value, _) => panic!("unexpected named mode {value}"),
        };
        // The reference processor may insert M73 after the template move.
        assert!(witness.starts_with(expected), "{}: {witness}", case.label);
        let moves = witness
            .lines()
            .filter(|line| line.starts_with("G1 "))
            .collect::<Vec<_>>();
        assert_eq!(moves, expected_moves, "{}", case.label);
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}
