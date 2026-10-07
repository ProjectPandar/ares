use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn vitrification_domain_covers_signed_endpoints_and_seeded_interior() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans
        .iter()
        .find(|p| p.key == "temperature_vitrification")
        .unwrap();
    let values = plan
        .cases
        .iter()
        .map(|case| {
            case.value.as_ref().unwrap()[0]
                .as_str()
                .unwrap()
                .parse::<i32>()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(values, [110]);
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
}

#[test]
fn orca_parity_vitrification_signed_values_reach_start_and_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|p| p.key == "temperature_vitrification")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut cases = plan.cases.clone();
    for value in [-123456789, i32::MAX] {
        cases.push(domains::OptionCase {
            label: value.to_string(),
            value: Some(json!([value.to_string()])),
        });
    }
    for case in &cases {
        let requested = case.value.as_ref().unwrap();
        let value = requested[0].as_str().unwrap();
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
        let start = machine.get_mut("machine_start_gcode").unwrap();
        *start = json!(format!(
            "{}\n;VITRIFICATION [min_vitrification_temperature]\n",
            start.as_str().unwrap()
        ));
        let exported = runner.export_case(
            &CaseInputs {
                label: &format!("vitrification/{value}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        );
        if value == "2147483647" {
            let error = exported
                .err()
                .expect("non-nullable INT_MAX must be rejected");
            assert_eq!(error.stage, runner::stages::Stage::Export);
            assert_eq!(error.kind, runner::stages::FailureKind::Process);
            let captured = error.captured.unwrap();
            assert!(String::from_utf8_lossy(&captured.stderr).contains("Serializing NaN"));
            assert!(error.evidence.is_some());
            continue;
        }
        let exported = exported.unwrap();
        let proof =
            runner::application::verify_value(&exported.project, &plan.key, requested).unwrap();
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], *requested);
        let sliced = runner.slice_case(&exported).unwrap();
        assert!(
            std::str::from_utf8(&sliced.reference)
                .unwrap()
                .lines()
                .any(|line| line == format!(";VITRIFICATION {value}"))
        );
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{value}: {}", outcome.detail);
    }
}
