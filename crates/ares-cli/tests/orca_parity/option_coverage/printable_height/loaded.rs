//! PrintConfig.cpp:779-787,10530-10564: scalar range, not the nullable vector.

use crate::runner::{self, CaseInputs, OrcaRunner};
use serde_json::json;
use std::io::{Cursor, Read, Write};

#[test]
fn printable_height_loaded_range_rejections() {
    if crate::native_environment::binary("ARES_ORCA_NATIVE_BIN").is_none()
        || crate::native_environment::value("ARES_ORCA_RESOURCES_DIR").is_none()
    {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = crate::presets::VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "printable-height/loaded-base",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let root = crate::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    // Config.cpp:321-337 accepts values within 1e-4 of the upper bound.
    for wire in ["-0.00001", "214700.0002"] {
        let mut reader = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..reader.len() {
            let mut file = reader.by_index(index).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if file.name() == "Metadata/project_settings.config" {
                let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                settings["printable_height"] = json!(wire);
                bytes = serde_json::to_vec(&settings).unwrap();
            }
            writer
                .start_file(file.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(&bytes).unwrap();
        }
        let project = writer.finish().unwrap().into_inner();
        runner::application::verify_value(&project, "printable_height", &json!(wire)).unwrap();
        let directory = tempfile::Builder::new()
            .prefix("printable-height-range-")
            .tempdir_in(&root)
            .unwrap()
            .keep();
        let input = directory.join("input.3mf");
        crate::artifacts::write(&input, &project).unwrap();
        let native = runner
            .early_cli_reference(&input, runner::EarlyExit::InvalidValues)
            .unwrap();
        for (name, bytes) in [
            ("native.stdout", &native.stdout),
            ("native.stderr", &native.stderr),
            ("native.result.json", &native.result_json),
        ] {
            crate::artifacts::write(&directory.join(name), bytes).unwrap();
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let error = runtime
            .block_on(ares_core::slice_project_plate_validated(
                &project,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                None,
                &ares_core::ConfigOverrides::default(),
                ares_core::SlicerContext::new(0),
            ))
            .unwrap_err();
        crate::artifacts::write(
            &directory.join("ares.validation.txt"),
            format!("{error:?}\n").as_bytes(),
        )
        .unwrap();
        let ares_core::SliceError::InvalidValuesInProject(message) = error else {
            panic!("{wire}: wrong public rejection {error:?}");
        };
        assert_eq!(
            native.stderr,
            format!("Param values in 3mf/config error: \n{message}\n").as_bytes()
        );
        let output = directory.join("plate_1.gcode");
        let cli = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
            .args(["slice", "-o"])
            .arg(&output)
            .arg(&input)
            .output()
            .unwrap();
        for (name, bytes) in [("ares.stdout", &cli.stdout), ("ares.stderr", &cli.stderr)] {
            crate::artifacts::write(&directory.join(name), bytes).unwrap();
        }
        assert_eq!(cli.status.code(), Some(if cfg!(unix) { 238 } else { -18 }));
        assert_eq!(cli.stdout, native.stdout);
        assert_eq!(cli.stderr, native.stderr);
        assert_eq!(
            crate::native_wall_clock::result_comparison_bytes(
                &std::fs::read(directory.join("result.json")).unwrap()
            ),
            crate::native_wall_clock::result_comparison_bytes(&native.result_json)
        );
        assert!(!output.exists());
        proofs.push(json!({"wire":wire,"validation":message,"code":cli.status.code(),"input":input,"Native_receipt":native.directory.join("receipt.json")}));
    }
    crate::artifacts::overwrite(
        &root.join("printable-height-loaded-range.json"),
        &serde_json::to_vec_pretty(&proofs).unwrap(),
    )
    .unwrap();
}
