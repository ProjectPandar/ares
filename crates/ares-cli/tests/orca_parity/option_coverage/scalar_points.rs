use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_scalar_point_probes_preserve_coordinates() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plans = domains::load(&runner::repo_root());
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    for (key, canonical, alternate) in [
        ("bed_mesh_min", "5,5", "5x5"),
        ("bed_mesh_max", "5,5", "5x5"),
        ("bed_mesh_probe_distance", "5,5", "5x5"),
        ("best_object_pos", "0.25,0.75", "0.25x0.75"),
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert_eq!(plan.cases.len(), 1);
        for alternate_input in [false, true] {
            let mut case = plan.cases[0].clone();
            if alternate_input {
                case.label = "alternate-x".into();
                case.value = Some(json!(alternate));
            }
            let mut machine = profiles.machine(&selection.printer).unwrap();
            let mut process = profiles.process(&selection.process).unwrap();
            parity::normalize_process_defaults(&machine, &mut process);
            let mut filaments = selection
                .filaments
                .iter()
                .map(|name| profiles.filament(name).unwrap())
                .collect::<Vec<_>>();
            parity::normalize_filament_defaults(&mut filaments);
            inject_case(plan, &case, &mut machine, &mut process, &mut filaments);
            let exported = runner
                .export_case(
                    &CaseInputs {
                        label: &format!("scalar-point/{key}/{}", case.label),
                        machine: &machine,
                        process: &process,
                        filaments: &filaments,
                    },
                    &parity::smoke_overrides(),
                    &runner::repo_root().join("tests/parity/cube10.stl"),
                )
                .unwrap();
            if !alternate_input {
                runner::application::verify_value(
                    &exported.project,
                    key,
                    case.value.as_ref().unwrap(),
                )
                .unwrap();
            }
            runner::application::verify_value(&exported.project, key, &json!(canonical)).unwrap();
            assert!(
                runner::application::verify_value(&exported.project, key, &json!("1,2")).is_err()
            );
            let sliced = runner.slice_case(&exported).unwrap();
            let expected = format!("; {key} = {canonical}");
            assert!(
                std::str::from_utf8(&sliced.reference)
                    .unwrap()
                    .lines()
                    .any(|line| line == expected)
            );
            let outcome = parity::compare_case(&sliced);
            assert_eq!(outcome.status, "PASS", "{key}: {}", outcome.detail);
        }
    }
}
