use super::*;

#[test]
fn normal_support_normalizes_organic_style_to_grid() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "1"),
        ("support_type", "normal(auto)"),
        ("support_angle", "0"),
        ("independent_support_layer_height", "1"),
        ("support_top_z_distance", "0.15"),
        ("support_bottom_z_distance", "0.2"),
        ("support_interface_top_layers", "3"),
        ("support_interface_bottom_layers", "-1"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let mut failures = Vec::new();
    for style in ["grid", "organic"] {
        process.insert("support_style".into(), json!(style));
        let label = format!("support-style/normal-auto/{style}");
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        for (key, value) in settings {
            runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
        }
        runner::application::verify_value(&exported.project, "support_style", &json!(style))
            .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        assert!(has_support_extrusion(
            std::str::from_utf8(&sliced.reference).unwrap()
        ));
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
