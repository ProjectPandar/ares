use super::*;
use crate::runner::CaseInputs;

#[test]
fn filename_format_domain_exercises_empty_literal_and_template_values() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans
        .iter()
        .find(|plan| plan.key == "filename_format")
        .unwrap();
    let values = plan
        .cases
        .iter()
        .map(|case| case.value.as_ref().and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        [
            Some(""),
            Some("ares-parity.gcode"),
            Some("{input_filename_base}_{filament_type[0]}.gcode")
        ]
    );
}

#[test]
fn orca_parity_filename_format_preserves_explicit_output_and_complete_gcode() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plans = domains::load(&runner::repo_root());
    let plan = plans
        .iter()
        .find(|plan| plan.key == "filename_format")
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
        inject_case(plan, case, &mut machine, &mut process, &mut filaments);
        let requested = case.value.as_ref().unwrap();
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("filename-format/{}", case.label),
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
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], *requested);
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let expected = format!("; filename_format = {}", requested.as_str().unwrap());
        assert!(
            reference.lines().any(|line| line == expected),
            "{}",
            case.label
        );
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}
