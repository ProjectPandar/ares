/// Native and Ares slice the same original STL with replacement profiles and
/// overrides. Independent settings-view completeness remains a separate contract.
#[test]
fn profile_settings_slice_matches_native_replacement_stl() {
    let Some(runner) = OrcaRunner::from_env()
        .unwrap() else {
        return;
    };
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
    let overrides_json = br#"{"layer_height":"0.3"}"#;
    let overrides = ares_core::ConfigOverrides::from_json(overrides_json).unwrap();

    let stl = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    std::fs::write(runner.work.join("stl-cube10.stl"), &stl).unwrap();
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
            &runner.work.join(format!("stl-{kind}.json")),
            &Value::Object(preset),
        );
    }
    // The STL entry's view for the original STL bytes.
    let view = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::profile_settings(
            &stl,
            ares_core::ProfileSliceRequest {
                filename: "stl-cube10.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
        ))
        .unwrap();
    write_json(
        &runner.work.join("stl-settings.json"),
        &serde_json::to_value(&view).unwrap(),
    );

    let override_fields: serde_json::Map<String, Value> =
        serde_json::from_slice(overrides_json).unwrap();

    let output = runner.work.join("stl-reference");
    std::fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            super::join_paths(
                [runner.work.join("stl-machine.json"), runner.work.join("stl-process.json")].iter(),
            ),
            "--load-filaments".to_owned(),
            runner.work.join("stl-filament.json").to_str().unwrap().to_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            runner.work.join("stl-cube10.stl").to_str().unwrap().to_owned(),
        ]
        .into_iter()
        .chain(override_fields.iter().flat_map(|(key, value)| {
            [format!("--{}", key.replace('_', "-")), value.as_str().unwrap().to_owned()]
        })),
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let reference = std::fs::read(output.join("plate_1.gcode")).unwrap();

    let actual = crate::runner::profile_entry::slice_flattened(
        (&stl, "stl-cube10.stl"),
        &[runner.work.join("stl-machine.json"), runner.work.join("stl-process.json")],
        &[runner.work.join("stl-filament.json")],
        &overrides,
        ares_core::Placement::Center,
    )
        .unwrap();
    std::fs::write(runner.work.join("stl-ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual).unwrap();
}
