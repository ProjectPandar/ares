/// Oracle E2E for `profile_settings`: the returned flat config is written
/// verbatim into a copy of the input archive's `project_settings.config`;
/// the native oracle slices that rewritten 3MF directly (no
/// `--load-settings`), while the Rust entry slices the original with
/// catalog/selection/overrides. Identical G-code proves the view carries
/// every key the slice consumes — any missing key would fall back to the
/// native default and diverge.
#[test]
fn profile_settings_view_round_trips_through_3mf() {
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
    let overrides = ares_core::ConfigOverrides::from_json(br#"{"layer_height":"0.3"}"#).unwrap();

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
    let view_json = serde_json::to_string(&view).unwrap();

    // Rewrite the archive's project_settings.config with the view.
    let rewritten = rewrite_project_settings(&original, view_json.as_bytes());
    let view_path = runner.work.join("view-rewritten.3mf");
    std::fs::write(&view_path, &rewritten).unwrap();

    // Native oracle: slice the rewritten 3MF directly.
    let output = runner.work.join("view-reference");
    std::fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            view_path.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let reference = std::fs::read(output.join("plate_1.gcode")).unwrap();

    // Rust entry: original archive + catalog/selection/override.
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &original,
            ares_core::ProfileSliceRequest {
                filename: "view.3mf",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(runner.work.join("view-ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual).unwrap();
}
