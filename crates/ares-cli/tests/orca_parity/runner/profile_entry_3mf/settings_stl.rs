/// Oracle E2E case 2: the STL entry's view (routed through the same overlay
/// machinery when overrides are present) round-trips through a 3MF whose
/// embedded config is that view; the native oracle slices the rewritten
/// archive directly while the Rust entry slices the original STL with
/// catalog/selection/override.
#[test]
fn profile_settings_view_round_trips_through_stl() {
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

    // A carrier 3MF with the same geometry, exported from the presets so the
    // archive structure is native; its embedded config is replaced below.
    let stl = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    std::fs::write(runner.work.join("stl-cube10.stl"), &stl).unwrap();
    let settings = super::join_paths(
        [
            runner.work.join("stl-machine.json"),
            runner.work.join("stl-process.json"),
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
            &runner.work.join(format!("stl-{kind}.json")),
            &Value::Object(preset),
        );
    }
    let carrier = runner.work.join("stl.3mf");
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            settings,
            "--load-filaments".to_owned(),
            runner
                .work
                .join("stl-filament.json")
                .to_str()
                .unwrap()
                .to_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--export-3mf".to_owned(),
            carrier.to_str().unwrap().to_owned(),
            runner
                .work
                .join("stl-cube10.stl")
                .to_str()
                .unwrap()
                .to_owned(),
        ],
        stages::Stage::Export,
        &runner.work,
    )
    .unwrap();
    let carrier_bytes = std::fs::read(&carrier).unwrap();

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
    let rewritten = rewrite_project_settings(
        &carrier_bytes,
        serde_json::to_string(&view).unwrap().as_bytes(),
    );
    let view_path = runner.work.join("stl-rewritten.3mf");
    std::fs::write(&view_path, &rewritten).unwrap();

    let output = runner.work.join("stl-reference");
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

    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &stl,
            ares_core::ProfileSliceRequest {
                filename: "stl-cube10.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::Center,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(runner.work.join("stl-ares.gcode"), &actual).unwrap();
    // The native 3MF path materializes `upward_compatible_machine` via
    // create-on-read (`OrcaSlicer.cpp:1769`) while STL --load-settings does
    // not (ARD-documented deviation). Pin the deviation before filtering:
    // exactly one native line, with an empty value, and none on the Ares
    // side — any other shape fails instead of being silently dropped.
    {
        let native_lines = reference
            .split_inclusive(|byte| *byte == b'\n')
            .filter(|line| line.starts_with(b"; upward_compatible_machine ="))
            .collect::<Vec<_>>();
        assert_eq!(
            native_lines,
            vec!["; upward_compatible_machine = \n".as_bytes()],
            "the deviation must stay a single empty native line"
        );
        assert!(
            !actual
                .split_inclusive(|byte| *byte == b'\n')
                .any(|line| line.starts_with(b"; upward_compatible_machine =")),
            "the STL entry never exports the key"
        );
    }
    let reference = reference
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| !line.starts_with(b"; upward_compatible_machine ="))
        .collect::<Vec<_>>()
        .concat();
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual).unwrap();
}
