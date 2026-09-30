use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn cut_value_domains_cover_boolean_nil_and_distance_endpoints() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for (key, expected, count) in [
        ("long_retractions_when_cut", &["0", "1"][..], 2),
        (
            "filament_long_retractions_when_cut",
            &["0", "1", "nil"][..],
            3,
        ),
        ("retraction_distances_when_cut", &["10", "18"][..], 3),
        (
            "filament_retraction_distances_when_cut",
            &["10", "18", "nil"][..],
            4,
        ),
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let values = plan
            .cases
            .iter()
            .map(|case| case.value.as_ref().unwrap()[0].as_str().unwrap())
            .collect::<Vec<_>>();
        for value in expected {
            assert!(values.contains(value), "{key}: missing {value}");
        }
        assert_eq!(values.len(), count, "{key}");
        if key.contains("distances") {
            let seeded = plan
                .cases
                .iter()
                .find(|case| case.label == "seeded")
                .unwrap();
            let value = seeded.value.as_ref().unwrap()[0]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap();
            assert!(value > 10.0 && value < 18.0, "{key}: {value}");
        }
    }
}

#[test]
fn orca_parity_cut_values_machine_boolean() {
    compare_values("long_retractions_when_cut");
}

#[test]
fn orca_parity_cut_values_machine_distance() {
    compare_values("retraction_distances_when_cut");
}

#[test]
fn orca_parity_cut_values_filament_boolean_and_nil() {
    compare_values("filament_long_retractions_when_cut");
}

#[test]
fn orca_parity_cut_values_filament_distance_and_nil() {
    compare_values("filament_retraction_distances_when_cut");
}

fn compare_values(key: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == key)
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let filament_key = key.starts_with("filament_");
    let cases = plan.cases.iter().map(|case| (case, "1")).chain(
        plan.cases
            .iter()
            .filter(|case| key == "filament_long_retractions_when_cut" && case.label == "nil")
            .map(|case| (case, "0")),
    );
    for (case, machine_boolean) in cases {
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
        if machine_boolean == "0" {
            machine.insert("long_retractions_when_cut".into(), json!(["0"]));
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("cut-values/{key}/{}/machine-{machine_boolean}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        runner::application::verify_value(&exported.project, key, case.value.as_ref().unwrap())
            .unwrap();
        for (dependency, value) in [
            (
                "enable_long_retraction_when_cut",
                json!(if filament_key { "2" } else { "1" }),
            ),
            ("long_retractions_when_cut", json!([machine_boolean])),
            ("retraction_distances_when_cut", json!(["12"])),
            ("filament_long_retractions_when_cut", json!(["1"])),
            ("filament_retraction_distances_when_cut", json!(["16"])),
            ("use_relative_e_distances", json!("1")),
        ] {
            if dependency != key {
                runner::application::verify_value(&exported.project, dependency, &value).unwrap();
            }
        }
        let wire = case.value.as_ref().unwrap()[0].as_str().unwrap();
        let enabled = if key.contains("long_retractions") && wire == "nil" {
            machine_boolean == "1"
        } else {
            !key.contains("long_retractions") || wire != "0"
        };
        let distance = if key.contains("distances") {
            if wire == "nil" { "12" } else { wire }
        } else if filament_key {
            "16"
        } else {
            "12"
        };
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let witness = reference
            .split_once(";CUT_GATE_BEGIN\n")
            .unwrap()
            .1
            .split_once(";CUT_GATE_END")
            .unwrap()
            .0;
        let header = format!(";CUT_GATE {enabled} {distance}");
        assert_eq!(
            witness.lines().next(),
            Some(header.as_str()),
            "{key}/{}",
            case.label
        );
        let moves = witness
            .lines()
            .filter(|line| line.starts_with("G1 "))
            .collect::<Vec<_>>();
        if enabled {
            let expected = format!("G1 E-{distance} F1800");
            assert_eq!(moves, [expected.as_str()], "{key}/{}", case.label);
        } else {
            assert!(moves.is_empty(), "{key}/{}: {moves:?}", case.label);
        }
        let outcome = parity::compare_case(&sliced);
        assert_eq!(
            outcome.status, "PASS",
            "{key}/{}: {}",
            case.label, outcome.detail
        );
    }
}
