#[test]
fn profile_entry_3mf_load_settings() {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "LH").unwrap();
    let machine_name = "LH Stinger 0.4 nozzle";
    let process_name = profiles.compatible_process(machine_name).unwrap();
    let filament_name = profiles.compatible_filament(machine_name).unwrap();
    let catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "LH")],
        "imports": [],
    });
    let selection = json!({
        "machine": machine_name, "process": process_name, "filaments": [filament_name],
    });
    write_json(&runner.work.join("selection.json"), &selection);
    write_json(&runner.work.join("catalog.json"), &catalog);
    for (kind, fields) in [
        ("machine", profiles.machine(machine_name).unwrap()),
        ("process", profiles.process(&process_name).unwrap()),
        ("filament", profiles.filament(&filament_name).unwrap()),
    ] {
        let mut preset = fields;
        preset.insert("type".to_owned(), json!(kind));
        preset.insert("from".to_owned(), json!("system"));
        preset.insert("instantiation".to_owned(), json!("true"));
        write_json(
            &runner.work.join(format!("{kind}.json")),
            &Value::Object(preset),
        );
    }

    let stl = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    std::fs::write(runner.work.join("cube10.stl"), &stl).unwrap();

    // Step 1: build the 3MF whose embedded config records the same presets
    // (`--export-3mf` writes full_config into project_settings.config).
    let settings = super::join_paths(
        [
            runner.work.join("machine.json"),
            runner.work.join("process.json"),
        ]
        .iter(),
    );
    let project_path = runner.work.join("profile-entry.3mf");
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            settings.clone(),
            "--load-filaments".to_owned(),
            runner
                .work
                .join("filament.json")
                .to_str()
                .unwrap()
                .to_owned(),
            "--arrange".to_owned(),
            "1".to_owned(),
            "--export-3mf".to_owned(),
            project_path.to_str().unwrap().to_owned(),
            runner.work.join("cube10.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Export,
        &runner.work,
    )
    .unwrap();
    let project = std::fs::read(&project_path).unwrap();

    // Step 2: native oracle slices the 3MF with the same selected presets.
    let native = |name: &str| {
        let output = runner.work.join(name);
        std::fs::create_dir(&output).unwrap();
        stages::run(
            &runner.bin,
            [
                "--load-settings".to_owned(),
                settings.clone(),
                "--load-filaments".to_owned(),
                runner
                    .work
                    .join("filament.json")
                    .to_str()
                    .unwrap()
                    .to_owned(),
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                project_path.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &runner.work,
        )
        .unwrap();
        std::fs::read(output.join("plate_1.gcode")).unwrap()
    };
    let first = native("reference-first");
    let repeat = native("reference-repeat");
    let normalize = |bytes: &[u8]| {
        crate::golden::normalize_one_generator_line(bytes, crate::golden::GeneratorKind::Orca)
            .unwrap()
    };
    assert_eq!(
        normalize(&first),
        normalize(&repeat),
        "native full-output repeat"
    );

    let actual = crate::runner::profile_entry::slice_flattened(
        (&project, "profile-entry.3mf"),
        &[runner.work.join("machine.json"), runner.work.join("process.json")],
        &[runner.work.join("filament.json")],
        &ares_core::ConfigOverrides::default(),
        ares_core::Placement::None,
    )
        .unwrap();
    std::fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    let comparison = crate::golden::compare_ordered_bytes_generator_only(&first, &actual);
    write_json(
        &runner.work.join("result.json"),
        &json!({
            "selection": selection,
            "native_repeat": "PASS",
            "rust_entry": "ares_core::slice_with_load_settings",
            "evidence": crate::artifacts::EVIDENCE,
            "status": if comparison.is_ok() { "PASS" } else { "DIVERGENT" },
            "difference": comparison.as_ref().err().map(ToString::to_string),
        }),
    );
    comparison.unwrap();
}
