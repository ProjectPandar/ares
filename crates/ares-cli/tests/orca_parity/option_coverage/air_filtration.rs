use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_air_filtration_master_gate() {
    compare_values("activate_air_filtration");
}

#[test]
fn orca_parity_air_filtration_printer_capability() {
    compare_values("support_air_filtration");
}

#[test]
fn orca_parity_air_filtration_during_gate() {
    compare_values("activate_air_filtration_during_print");
}

#[test]
fn orca_parity_air_filtration_completion_gate() {
    compare_values("activate_air_filtration_on_completion");
}

#[test]
fn orca_parity_air_filtration_during_speed() {
    compare_values("during_print_exhaust_fan_speed");
}

#[test]
fn orca_parity_air_filtration_completion_speed() {
    compare_values("complete_print_exhaust_fan_speed");
}

#[test]
fn air_filtration_does_not_select_support_geometry() {
    assert!(!support_geometry_option("support_air_filtration"));
}

#[test]
fn air_filtration_domains_cover_all_gates_and_speed_boundaries() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in [
        "support_air_filtration",
        "activate_air_filtration",
        "activate_air_filtration_during_print",
        "activate_air_filtration_on_completion",
        "during_print_exhaust_fan_speed",
        "complete_print_exhaust_fan_speed",
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let values = plan
            .cases
            .iter()
            .map(|case| {
                let value = case.value.as_ref().unwrap();
                value.as_str().unwrap_or_else(|| value[0].as_str().unwrap())
            })
            .collect::<Vec<_>>();
        if key.ends_with("speed") {
            assert_eq!(values.len(), 3, "{key}");
            assert_eq!(&values[..2], ["0", "100"], "{key}");
            let interior = values[2].parse::<u32>().unwrap();
            assert!((1..100).contains(&interior), "{key}: {interior}");
        } else {
            assert_eq!(values, ["0", "1"], "{key}");
        }
    }
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
                    label: &format!("air-filtration/{key}/{}", case.label),
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
            ("support_air_filtration", json!("1")),
            ("activate_air_filtration", json!(["1"])),
            ("activate_air_filtration_during_print", json!(["1"])),
            ("activate_air_filtration_on_completion", json!(["1"])),
            ("during_print_exhaust_fan_speed", json!(["40"])),
            ("complete_print_exhaust_fan_speed", json!(["80"])),
            ("enable_support", json!("0")),
        ] {
            if dependency != key {
                runner::application::verify_value(&exported.project, dependency, &value).unwrap();
            }
        }
        let value = case.value.as_ref().unwrap();
        let wire = value.as_str().unwrap_or_else(|| value[0].as_str().unwrap());
        let disabled = wire == "0";
        let all_off =
            disabled && matches!(key, "support_air_filtration" | "activate_air_filtration");
        let mut expected = Vec::new();
        for (gate, speed, baseline) in [
            (
                "activate_air_filtration_during_print",
                "during_print_exhaust_fan_speed",
                40,
            ),
            (
                "activate_air_filtration_on_completion",
                "complete_print_exhaust_fan_speed",
                80,
            ),
        ] {
            if all_off || key == gate && disabled {
                continue;
            }
            let percent = if key == speed {
                wire.parse::<u32>().unwrap()
            } else {
                baseline
            };
            expected.push(format!("M106 P3 S{}", percent * 255 / 100));
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let commands = reference
            .lines()
            .filter(|line| line.starts_with("M106 P3 "))
            .collect::<Vec<_>>();
        assert_eq!(commands, expected, "{key}/{}", case.label);
        let outcome = parity::compare_case(&sliced);
        assert_eq!(
            outcome.status, "PASS",
            "{key}/{}: {}",
            case.label, outcome.detail
        );
    }
}
