use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn ironing_fan_preserves_complete_output_and_role_speed() {
    if !crate::runner::reference_enabled("ARES_PARITY_IRONING_FAN_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("ironing_type".into(), json!("solid"));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let mut failures = Vec::new();
    for (label, speed, closed, ramp, overhang, expected_pwm) in [
        ("disabled", "-1", "0", "0", "0", 255),
        ("off", "0", "0", "0", "0", 0),
        ("below-baseline", "45", "0", "0", "0", 114),
        ("closed-layers", "100", "1000", "0", "0", 0),
        ("overhang-enabled", "45", "0", "0", "1", 114),
        ("fixed-during-ramp", "45", "0", "1000", "0", 114),
    ] {
        let settings = [
            ("ironing_fan_speed", speed),
            ("close_fan_the_first_x_layers", closed),
            ("full_fan_speed_layer", ramp),
            ("enable_overhang_bridge_fan", overhang),
            ("fan_min_speed", "100"),
            ("fan_max_speed", "100"),
            ("reduce_fan_stop_start_freq", "1"),
        ];
        for filament in &mut filaments {
            for (key, value) in settings {
                filament.insert(key.into(), json!([value]));
            }
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("ironing-fan/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        runner::application::verify_value(&exported.project, "ironing_type", &json!("solid"))
            .unwrap();
        for (key, value) in settings {
            runner::application::verify_value(&exported.project, key, &json!([value])).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        assert_eq!(
            ironing_fan_speeds(std::str::from_utf8(&sliced.reference).unwrap()),
            [expected_pwm].into(),
            "native role witness: {label}"
        );
        let outcome = parity::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn ironing_fan_speeds(reference: &str) -> std::collections::BTreeSet<i32> {
    let mut speeds = std::collections::BTreeSet::new();
    let mut ironing = false;
    let mut resumed = false;
    let mut relative_e = false;
    let mut fan = 0;
    for line in reference.lines() {
        if let Some(role) = line
            .strip_prefix(";TYPE:")
            .or_else(|| line.strip_prefix("; FEATURE: "))
        {
            ironing = role == "Ironing";
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        match words.clone().next() {
            Some("M83") => relative_e = true,
            Some("M82") => relative_e = false,
            Some("M107") => fan = 0,
            Some("M106") if !words.clone().any(|word| word.starts_with('P')) => {
                fan = words
                    .find_map(|word| word.strip_prefix('S'))
                    .unwrap()
                    .parse()
                    .unwrap();
            }
            Some("G1")
                if relative_e
                    && words.clone().any(|word| word.starts_with(['X', 'Y']))
                    && words.clone().any(|word| {
                        word.strip_prefix('E')
                            .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
                    }) =>
            {
                if ironing {
                    speeds.insert(fan);
                } else {
                    resumed |= !speeds.is_empty();
                }
            }
            _ => {}
        }
    }
    assert!(!speeds.is_empty(), "native must extrude ironing paths");
    assert!(resumed, "native must extrude another role after ironing");
    speeds
}
