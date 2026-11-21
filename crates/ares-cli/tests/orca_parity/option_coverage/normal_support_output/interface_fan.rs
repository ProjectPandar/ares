use super::*;

#[test]
fn support_interface_fan_preserves_complete_output_and_role_speed() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let support_settings = [
        ("enable_support", "1"),
        ("support_type", "normal(auto)"),
        ("support_style", "grid"),
        ("independent_support_layer_height", "1"),
        ("support_top_z_distance", "0.15"),
        ("support_bottom_z_distance", "0.2"),
        ("support_interface_top_layers", "3"),
        ("support_interface_bottom_layers", "-1"),
    ];
    for (key, value) in support_settings {
        process.insert(key.into(), json!(value));
    }
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
        let fan_settings = [
            ("support_material_interface_fan_speed", speed),
            ("close_fan_the_first_x_layers", closed),
            ("full_fan_speed_layer", ramp),
            ("enable_overhang_bridge_fan", overhang),
            ("fan_min_speed", "100"),
            ("fan_max_speed", "100"),
            ("reduce_fan_stop_start_freq", "1"),
        ];
        for filament in &mut filaments {
            for (key, value) in fan_settings {
                filament.insert(key.into(), json!([value]));
            }
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("normal-support/interface-fan/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        for (key, value) in support_settings {
            runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
        }
        for (key, value) in fan_settings {
            runner::application::verify_value(&exported.project, key, &json!([value])).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let speeds = interface_fan_speeds(std::str::from_utf8(&sliced.reference).unwrap());
        assert_eq!(
            speeds,
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

fn interface_fan_speeds(reference: &str) -> std::collections::BTreeSet<i32> {
    let mut speeds = std::collections::BTreeSet::new();
    let mut interface = false;
    let mut relative_e = false;
    let mut fan = 0;
    for line in reference.lines() {
        if let Some(role) = line
            .strip_prefix(";TYPE:")
            .or_else(|| line.strip_prefix("; FEATURE: "))
        {
            interface = role == "Support interface";
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
                if interface
                    && relative_e
                    && words.clone().any(|word| word.starts_with(['X', 'Y']))
                    && words.clone().any(|word| {
                        word.strip_prefix('E')
                            .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
                    }) =>
            {
                speeds.insert(fan);
            }
            _ => {}
        }
    }
    assert!(!speeds.is_empty(), "native must extrude support interface");
    speeds
}
