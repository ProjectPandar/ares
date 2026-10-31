use super::*;

#[test]
fn interlaced_support_cooling_and_equalizer_preserve_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_NORMAL_SUPPORT_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("auxiliary_fan".into(), json!("1"));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_support", "1"),
        ("support_type", "normal(auto)"),
        ("support_style", "grid"),
        ("independent_support_layer_height", "1"),
        ("support_top_z_distance", "0.15"),
        ("support_bottom_z_distance", "0.2"),
        ("support_interface_top_layers", "3"),
        ("support_interface_bottom_layers", "-1"),
    ] {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    for filament in &mut filaments {
        for (key, value) in [
            ("close_fan_the_first_x_layers", "2"),
            ("full_fan_speed_layer", "6"),
            ("additional_cooling_fan_speed", "70"),
            ("overhang_fan_speed", "100"),
            ("enable_overhang_bridge_fan", "1"),
        ] {
            filament.insert(key.into(), json!([value]));
        }
    }
    let mut failures = Vec::new();
    for (slope, close_layer, full_layer, overhang_speed, baseline) in [
        ("0", "2", "6", "100", "100"),
        ("2", "2", "6", "100", "100"),
        ("0", "1", "11", "45", "0"),
    ] {
        let mut candidate_filaments = filaments.clone();
        for filament in &mut candidate_filaments {
            for (key, value) in [
                ("close_fan_the_first_x_layers", close_layer),
                ("full_fan_speed_layer", full_layer),
                ("overhang_fan_speed", overhang_speed),
                ("fan_min_speed", baseline),
                ("fan_max_speed", baseline),
            ] {
                filament.insert(key.into(), json!([value]));
            }
        }
        process.insert("max_volumetric_extrusion_rate_slope".into(), json!(slope));
        let label = format!(
            "normal-support/cooling/aux-role-fan/equalizer-{slope}/ramp-{full_layer}-{overhang_speed}"
        );
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &candidate_filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        for (key, value) in [
            ("enable_support", json!("1")),
            ("auxiliary_fan", json!("1")),
            ("max_volumetric_extrusion_rate_slope", json!(slope)),
            ("close_fan_the_first_x_layers", json!([close_layer])),
            ("full_fan_speed_layer", json!([full_layer])),
            ("overhang_fan_speed", json!([overhang_speed])),
            ("fan_min_speed", json!([baseline])),
            ("fan_max_speed", json!([baseline])),
            ("additional_cooling_fan_speed", json!(["70"])),
        ] {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let witness = has_support_extrusion(reference)
            && reference.contains("M106 P2 S178")
            && (overhang_speed != "45" || reference.contains("M106 S81\n"));
        if outcome.status != "PASS" || !witness {
            failures.push(format!(
                "{label}: {} (support/aux witness={witness})",
                outcome.detail
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
