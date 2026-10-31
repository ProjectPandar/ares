//! Selected-profile slicing over a Bambu/Orca 3MF: the embedded
//! `project_settings.config` is the full-config base and the loaded presets
//! overlay only the keys they supply
//! (`OrcaSlicer.cpp:1624-1640,2750-2812,2849-2867,2993-3011,3115-3337`).
//! Native reference: `--export-3mf` followed by `3mf --load-settings` on the
//! same flattened presets; the Rust side receives the raw vendor catalog.

use serde_json::{Value, json};

use super::{
    OrcaRunner,
    profile_entry::{placed_cube, vendor, write_json},
    stages,
};

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

    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &project,
            ares_core::ProfileSliceRequest {
                filename: "profile-entry.3mf",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &ares_core::ConfigOverrides::default(),
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    let comparison = crate::golden::compare_ordered_bytes_generator_only(&first, &actual);
    write_json(
        &runner.work.join("result.json"),
        &json!({
            "selection": selection,
            "native_repeat": "PASS",
            "rust_entry": "ares_core::slice_with_profiles",
            "evidence": crate::artifacts::EVIDENCE,
            "status": if comparison.is_ok() { "PASS" } else { "DIVERGENT" },
            "difference": comparison.as_ref().err().map(ToString::to_string),
        }),
    );
    comparison.unwrap();
}

/// Override parity on the 3MF entry: the native oracle receives the same
/// override written into the `--load-settings` process JSON, while the Rust
/// entry slices the pristine catalog with the override document (M854 edit
/// order: preset → user edit wins, `Config.cpp:461`). Full ordered output
/// must be byte-identical (generator line normalized).
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
    // The exported 3MF still embeds the PRISTINE process: both sides start
    // from the same archive and only the loaded/override value differs in
    // name, not in effect.
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            settings.clone(),
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

    let output = runner.work.join("override-reference");
    std::fs::create_dir(&output).unwrap();
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

/// Rebuilds the 3MF with `Metadata/project_settings.config` replaced by
/// `settings`; every other member is copied verbatim.
fn rewrite_project_settings(archive_bytes: &[u8], settings: &[u8]) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(std::io::Cursor::new(archive_bytes)).unwrap();
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_owned();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut bytes).unwrap();
        writer.start_file(&name, options).unwrap();
        if name.eq_ignore_ascii_case("Metadata/project_settings.config") {
            std::io::Write::write_all(&mut writer, settings).unwrap();
        } else {
            std::io::Write::write_all(&mut writer, &bytes).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

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

/// O5 real-case oracle: the MK3S fixture (exported from the Prusa
/// `process_common_mk3` chain carrying `post_process: [""]`) is rejected
/// by the native CLI with exit code -19 and by Ares with the same message.
#[test]
fn mk3s_post_process_rejection() {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = super::repo_root();
    let fixture =
        std::fs::read(root.join("tests/parity/validation/post-process-mk3s-postprocess.3mf"))
            .unwrap();
    std::fs::write(runner.work.join("o5-mk3s.3mf"), &fixture).unwrap();

    // Native: direct 3MF slice → exit -19 (CLI_POSTPROCESS_NOT_SUPPORTED).
    let output = runner.work.join("o5-reference");
    std::fs::create_dir(&output).unwrap();
    let result = std::fs::read(runner.work.join("o5-mk3s.3mf")).unwrap();
    let native = stages::run(
        &runner.bin,
        [
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            runner.work.join("o5-mk3s.3mf").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    );
    // The native exits 237 (= 256 - 19); assert rejection, not success.
    assert!(native.is_err(), "native must reject the post_process 3MF");

    // Ares: the validated entry returns the exact upstream message.
    let error = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &result,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("post_process is not supported under CLI."),
        "{error}"
    );
    // No G-code was produced.
    assert!(!output.join("plate_1.gcode").exists());
}
