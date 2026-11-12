/// Native and Ares slice the same original 3MF with replacement profiles and
/// overrides. Independent settings-view completeness remains a separate contract.
#[test]
fn profile_settings_slice_matches_native_replacement_3mf() {
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
    let overrides_json = br#"{"layer_height":"0.3"}"#;
    let overrides = ares_core::ConfigOverrides::from_json(overrides_json).unwrap();

    // Original 3MF: embedded config from the pristine presets.
    let stl = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    std::fs::write(runner.work.join("cube10.stl"), &stl).unwrap();
    let settings = super::join_paths(
        [
            runner.work.join("view-machine.json"),
            runner.work.join("view-process.json"),
        ]
        .iter(),
    );
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
            &runner.work.join(format!("view-{kind}.json")),
            &Value::Object(preset),
        );
    }
    let project_path = runner.work.join("view.3mf");
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            settings,
            "--load-filaments".to_owned(),
            runner
                .work
                .join("view-filament.json")
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
    let original = std::fs::read(&project_path).unwrap();

    // The effective view for the original + catalog + selection + override.
    let view = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::profile_settings(
            &original,
            ares_core::ProfileSliceRequest {
                filename: "view.3mf",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
        ))
        .unwrap();
    write_json(
        &runner.work.join("view-settings.json"),
        &serde_json::to_value(&view).unwrap(),
    );

    let override_fields: serde_json::Map<String, Value> =
        serde_json::from_slice(overrides_json).unwrap();

    // Native uses the same replacement inputs and overrides as the selection.
    let output = runner.work.join("view-reference");
    std::fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            super::join_paths(
                [runner.work.join("view-machine.json"), runner.work.join("view-process.json")].iter(),
            ),
            "--load-filaments".to_owned(),
            runner.work.join("view-filament.json").to_str().unwrap().to_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            project_path.to_str().unwrap().to_owned(),
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

    // Rust loads the same flattened replacement files and CLI overrides as Native.
    let actual = crate::runner::profile_entry::slice_flattened(
        (&original, "view.3mf"),
        &[runner.work.join("view-machine.json"), runner.work.join("view-process.json")],
        &[runner.work.join("view-filament.json")],
        &overrides,
        ares_core::Placement::None,
    )
        .unwrap();
    std::fs::write(runner.work.join("view-ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual).unwrap();
}
