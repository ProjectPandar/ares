use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_pressure_advance_boundaries_emit_commands() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        return;
    };
    let mut plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "pressure_advance")
        .unwrap();
    assert!(
        plan.cases
            .iter()
            .any(|case| case.value == Some(json!(["2"])))
    );
    for (label, value) in [("zero", "0"), ("negative", "-1")] {
        plan.cases.push(domains::OptionCase {
            label: label.into(),
            value: Some(json!([value])),
        });
    }
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
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("pressure-advance/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        for (key, value) in [
            ("pressure_advance", case.value.clone().unwrap()),
            ("enable_pressure_advance", json!(["1"])),
            ("adaptive_pressure_advance", json!(["0"])),
        ] {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let command = reference.lines().find(|line| line.starts_with("M900 K"));
        match case.label.as_str() {
            "zero" => assert_eq!(command, Some("M900 K0; Override pressure advance value")),
            "max" => assert_eq!(command, Some("M900 K2; Override pressure advance value")),
            "negative" => assert!(command.is_none()),
            _ => assert!(command.is_some()),
        }
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}
