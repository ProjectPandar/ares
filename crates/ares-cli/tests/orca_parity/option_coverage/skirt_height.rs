use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_skirt_height_controls_printed_layers() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        return;
    };
    let mut plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "skirt_height")
        .unwrap();
    for (label, value) in [("negative", "-1"), ("one", "1"), ("two", "2")] {
        plan.cases.push(domains::OptionCase {
            label: label.into(),
            value: Some(json!(value)),
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
        let exported = runner.export_case(
            &CaseInputs {
                label: &format!("skirt-height/{}", case.label),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        );
        if case.label == "negative" {
            match exported {
                Err(error) => assert!(error.input_rejection("skirt_height"), "{error}"),
                Ok(_) => panic!("negative skirt height must be rejected by Orca"),
            }
            continue;
        }
        let exported = exported.unwrap();
        for (key, value) in [
            ("skirt_height", case.value.clone().unwrap()),
            ("skirt_loops", json!("1")),
            ("draft_shield", json!("disabled")),
        ] {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let layers = reference.split(";LAYER_CHANGE").skip(1).collect::<Vec<_>>();
        assert_eq!(layers.len(), 50, "10 mm cube at 0.2 mm layer height");
        let skirt_layers = layers
            .iter()
            .filter(|layer| layer.contains(";TYPE:Skirt\n"))
            .count();
        let expected = match case.label.as_str() {
            "min" => 0,
            "one" => 1,
            "two" => 2,
            "max" | "seeded" => 50,
            label => panic!("unexpected skirt coverage case {label}"),
        };
        assert_eq!(skirt_layers, expected, "{}", case.label);
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}
