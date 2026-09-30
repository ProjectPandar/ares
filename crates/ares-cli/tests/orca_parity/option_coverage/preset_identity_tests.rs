use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

const KEYS: [&str; 3] = ["filament_ids", "filament_settings_id", "print_settings_id"];

#[test]
fn preset_identity_activation_preserves_requested_values_and_unrelated_fields() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in KEYS {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let mut machine = Map::new();
        let mut process = json!({"name":"original-process", "layer_height":"0.2"})
            .as_object()
            .unwrap()
            .clone();
        let mut filaments = vec![
            json!({
                "name":"original-filament", "filament_id":"original-id", "filament_type":["PLA"]
            })
            .as_object()
            .unwrap()
            .clone(),
        ];
        let case = &plan.cases[0];
        inject_case(plan, case, &mut machine, &mut process, &mut filaments);
        match plan.key.as_str() {
            "filament_ids" => assert_eq!(filaments[0]["filament_id"], "ares-parity-probe"),
            "filament_settings_id" => assert_eq!(filaments[0]["name"], "ares-parity-probe"),
            "print_settings_id" => assert_eq!(process["name"], "ares-parity-probe"),
            _ => unreachable!(),
        }
        assert_eq!(process["layer_height"], "0.2");
        assert_eq!(filaments[0]["filament_type"], json!(["PLA"]));
        assert_eq!(
            process
                .get(&plan.key)
                .or_else(|| filaments[0].get(&plan.key)),
            case.value.as_ref()
        );
    }
}

#[test]
fn orca_parity_preset_identities_reach_export_and_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plans = domains::load(&runner::repo_root());
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    for key in KEYS {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let mut cases = plan.cases.clone();
        cases.push(domains::OptionCase {
            label: "second-identity".into(),
            value: Some(if plan.option_type == "coStrings" {
                json!(["ares-parity-probe-2"])
            } else {
                json!("ares-parity-probe-2")
            }),
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
            inject_case(plan, case, &mut machine, &mut process, &mut filaments);
            let requested = case.value.as_ref().unwrap();
            let exported = runner
                .export_case(
                    &CaseInputs {
                        label: &format!("preset-identity/{}/{}", plan.key, case.label),
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
            let value = requested
                .as_str()
                .unwrap_or_else(|| requested[0].as_str().unwrap());
            let sliced = runner.slice_case(&exported).unwrap();
            let reference = std::str::from_utf8(&sliced.reference).unwrap();
            assert!(
                reference
                    .lines()
                    .any(|line| line == format!("; {} = {value}", plan.key))
            );
            let outcome = parity::compare_case(&sliced);
            assert_eq!(outcome.status, "PASS", "{}: {}", plan.key, outcome.detail);
        }
    }
}
