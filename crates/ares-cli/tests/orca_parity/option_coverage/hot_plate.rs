use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_hot_plate_temperature_controls_heating() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        return;
    };
    let mut plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "hot_plate_temp_initial_layer")
        .unwrap();
    for (label, value) in [("negative", "-1"), ("zero", "0"), ("positive", "55")] {
        plan.cases.push(domains::OptionCase {
            label: label.into(),
            value: Some(json!([value])),
        });
    }
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    for startup in ["automatic", "vendor"] {
        for formula in ["by_first_filament", "by_highest_temp"] {
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
                machine.insert("bed_temperature_formula".into(), json!(formula));
                if startup == "automatic" {
                    machine.insert("machine_start_gcode".into(), json!(""));
                }
                let exported = runner
                    .export_case(
                        &CaseInputs {
                            label: &format!("hot-plate/{startup}/{formula}/{}", case.label),
                            machine: &machine,
                            process: &process,
                            filaments: &filaments,
                        },
                        &parity::smoke_overrides(),
                        &runner::repo_root().join("tests/parity/cube10.stl"),
                    )
                    .unwrap();
                let value = case.value.as_ref().unwrap();
                runner::application::verify_value(&exported.project, &plan.key, value).unwrap();
                runner::application::verify_value(
                    &exported.project,
                    "curr_bed_type",
                    &json!("High Temp Plate"),
                )
                .unwrap();
                runner::application::verify_value(
                    &exported.project,
                    "bed_temperature_formula",
                    &json!(formula),
                )
                .unwrap();
                let sliced = runner.slice_case(&exported).unwrap();
                let reference = std::str::from_utf8(&sliced.reference).unwrap();
                let temperature = value[0].as_str().unwrap().parse::<i32>().unwrap();
                let temperature = if formula == "by_highest_temp" {
                    temperature.max(0)
                } else {
                    temperature
                };
                let commands: &[&str] = if startup == "automatic" {
                    &["M190"]
                } else {
                    &["M140", "M190"]
                };
                for command in commands {
                    let expected = format!("{command} S{temperature}");
                    let actual = reference.lines().find(|line| line.starts_with(command));
                    if startup == "automatic" && temperature == 0 {
                        assert!(actual.is_none());
                        continue;
                    }
                    assert_eq!(
                        actual.map(|line| line.split(';').next().unwrap().trim()),
                        Some(expected.as_str())
                    );
                }
                let outcome = parity::compare_case(&sliced);
                assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
            }
        }
    }
}
