use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_obsolete_silent_mode_values_export_false_and_match_full_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "silent_mode")
        .unwrap();
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone())
            .collect::<Vec<_>>(),
        [Some(json!("0")), Some(json!("1"))]
    );
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
        let requested = case.value.as_ref().unwrap();
        assert_eq!(&machine["silent_mode"], requested);
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("obsolete-silent-mode/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let proof =
            runner::application::verify_value(&exported.project, "silent_mode", requested).unwrap();
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], "0");
        assert_eq!(proof["normalization"]["kind"], "obsolete_input");
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(reference.lines().any(|line| line == "; silent_mode = 0"));
        assert!(!reference.lines().any(|line| line.starts_with("M73 Q")));
        assert!(!reference.contains("estimated printing time (silent mode)"));
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}
