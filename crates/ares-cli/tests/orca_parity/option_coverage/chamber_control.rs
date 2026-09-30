use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_chamber_capability_is_not_a_runtime_gate() {
    compare_values("support_chamber_temp_control", None);
}

#[test]
fn orca_parity_chamber_activation_controls_heating_and_shutdown() {
    compare_values("activate_chamber_temp_control", None);
}

#[test]
fn orca_parity_chamber_target_covers_zero_maximum_and_interior() {
    compare_values("chamber_temperature", None);
}

#[test]
fn orca_parity_chamber_minimum_reaches_the_start_template() {
    compare_values("chamber_minimal_temperature", None);
}

#[test]
fn orca_parity_chamber_auxiliary_fan_brackets_waiting_heat() {
    compare_values("auxiliary_fan", None);
}

#[test]
fn orca_parity_chamber_custom_start_suppresses_only_automatic_heating() {
    for command in ["M141", "M191"] {
        compare_values("activate_chamber_temp_control", Some(command));
    }
}

#[test]
fn chamber_control_domains_cover_gates_and_temperature_endpoints() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in [
        "support_chamber_temp_control",
        "activate_chamber_temp_control",
        "chamber_temperature",
        "chamber_minimal_temperature",
        "auxiliary_fan",
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
        if key.ends_with("temperature") {
            assert_eq!(values.len(), 3, "{key}");
            assert_eq!(&values[..2], ["0", "1500"], "{key}");
            assert!(
                (1..1500).contains(&values[2].parse::<u32>().unwrap()),
                "{key}"
            );
        } else {
            assert_eq!(values, ["0", "1"], "{key}");
        }
    }
}

#[test]
fn chamber_capability_is_not_an_unimplemented_support_generator() {
    let case = domains::OptionCase {
        label: "true".into(),
        value: Some(json!("1")),
    };
    assert!(!support_geometry_option("support_chamber_temp_control"));
    assert!(!sweep::unimplemented_feature(
        "support_chamber_temp_control",
        &case
    ));
}

fn compare_values(key: &str, custom: Option<&str>) {
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
        if custom.is_some() && case.label != "true" {
            continue;
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
        inject_case(&plan, case, &mut machine, &mut process, &mut filaments);
        if let Some(command) = custom {
            let start = machine.get_mut("machine_start_gcode").unwrap();
            *start = json!(format!("{}\n{command} S37\n", start.as_str().unwrap()));
        }
        let target = if key == "chamber_minimal_temperature" {
            "1500"
        } else {
            "60"
        };
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!(
                        "chamber/{key}/{}/{}",
                        case.label,
                        custom.unwrap_or("automatic")
                    ),
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
            ("support_chamber_temp_control", json!("1")),
            ("activate_chamber_temp_control", json!(["1"])),
            ("chamber_temperature", json!([target])),
            ("chamber_minimal_temperature", json!(["0"])),
            ("auxiliary_fan", json!("0")),
            ("enable_support", json!("0")),
        ] {
            if dependency != key {
                runner::application::verify_value(&exported.project, dependency, &value).unwrap();
            }
        }
        let value = case.value.as_ref().unwrap();
        let wire = value.as_str().unwrap_or_else(|| value[0].as_str().unwrap());
        let temperature = if key == "chamber_temperature" {
            wire
        } else {
            target
        };
        let enabled = match key {
            "activate_chamber_temp_control" => wire == "1",
            "chamber_temperature" => wire != "0",
            _ => true,
        };
        let auxiliary = key == "auxiliary_fan" && wire == "1";
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let minimum = if key == "chamber_minimal_temperature" {
            wire
        } else {
            "0"
        };
        let minimum_witness = format!(";CHAMBER_MIN {minimum}");
        assert!(reference.lines().any(|line| line == minimum_witness));
        let commands = reference
            .lines()
            .filter(|line| line.starts_with("M141 ") || line.starts_with("M191 "))
            .collect::<Vec<_>>();
        let mut expected = Vec::new();
        if let Some(command) = custom {
            expected.push(format!("{command} S37"));
        } else if enabled {
            expected.push(format!(
                "M191 S{temperature} ;set chamber_temperature and wait for it to be reached"
            ));
        }
        if enabled {
            expected.push("M141 S0;set chamber_temperature".to_owned());
        }
        assert_eq!(commands, expected, "{key}/{}", case.label);
        let startup = reference
            .split_once(";CHAMBER_TEMPLATE_BEGIN")
            .unwrap()
            .0
            .rsplit_once(";TYPE:Custom\n")
            .unwrap()
            .1;
        let fan = startup
            .lines()
            .filter(|line| line.starts_with("M106 P2 "))
            .collect::<Vec<_>>();
        let expected_fan = if auxiliary && enabled && custom.is_none() {
            vec!["M106 P2 S255 ", "M106 P2 S0 "]
        } else {
            vec![]
        };
        assert_eq!(fan, expected_fan, "{key}/{}", case.label);
        let outcome = parity::compare_case(&sliced);
        assert_eq!(
            outcome.status, "PASS",
            "{key}/{}: {}",
            case.label, outcome.detail
        );
    }
}
