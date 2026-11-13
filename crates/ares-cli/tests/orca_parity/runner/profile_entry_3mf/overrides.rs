/// Both 3MF entries load the pristine vendor presets before applying the
/// layer-height edit (`Config.cpp:461`, `OrcaSlicer.cpp:4138-4226`). Full ordered
/// output must be byte-identical (generator line normalized).
#[test]
fn profile_entry_3mf_override_parity() {
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
    let mut overridden_process = profiles.process(&process_name).unwrap();
    // The same edit the override applies: write it into the native process
    // JSON so upstream's own application path produces the reference.
    overridden_process.insert("layer_height".to_owned(), json!("0.3"));
    for (kind, fields) in [
        ("machine", profiles.machine(machine_name).unwrap()),
        ("process", overridden_process),
        ("filament", profiles.filament(&filament_name).unwrap()),
    ] {
        let mut preset = fields;
        preset.insert("type".to_owned(), json!(kind));
        preset.insert("from".to_owned(), json!("system"));
        preset.insert("instantiation".to_owned(), json!("true"));
        write_json(
            &runner.work.join(format!("override-{kind}.json")),
            &Value::Object(preset),
        );
    }

    let stl = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    std::fs::write(runner.work.join("cube10.stl"), &stl).unwrap();
    let settings = super::join_paths(
        [
            runner.work.join("override-machine.json"),
            runner.work.join("override-process.json"),
        ]
        .iter(),
    );
    let project_path = runner.work.join("override.3mf");
    // Both entries start from the same archive, exported with layer height 0.3.
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            settings,
            "--load-filaments".to_owned(),
            runner
                .work
                .join("override-filament.json")
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
    let native_presets = crate::runner::profile_entry::native_user_presets::resolved(
        &runner.work,
        &profiles_root,
        &catalog,
        &selection,
    );

    let output = runner.work.join("override-reference");
    std::fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--datadir".to_owned(),
            native_presets.datadir.to_str().unwrap().to_owned(),
            "--load-settings".to_owned(),
            super::join_paths(native_presets.settings.iter()),
            "--load-filaments".to_owned(),
            super::join_paths(native_presets.filaments.iter()),
            "--layer-height".to_owned(),
            "0.3".to_owned(),
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
    let reference = std::fs::read(output.join("plate_1.gcode")).unwrap();

    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &project,
            ares_core::ProfileSliceRequest {
                filename: "override.3mf",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &ares_core::ConfigOverrides::from_json(br#"{"layer_height":"0.3"}"#)
                    .unwrap(),
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(runner.work.join("override-ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual).unwrap();
}
