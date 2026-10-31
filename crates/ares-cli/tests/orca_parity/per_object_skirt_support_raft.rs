//! Public RED for per-object skirts with support/raft footprints and layers.
//! Upstream: Print.cpp:2669-2696; GCode.cpp:4311-4318,4398,4455-4485,5329-5335.

use crate::{CaseInputs, OrcaRunner, golden, normalize_process_defaults, runner, select_printer};
use serde_json::json;
use sha2::Digest;
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

#[test]
fn per_object_skirt_control_matches_native() {
    run_case("control", false, 0, false);
}

#[test]
fn per_object_skirt_single_raft_control_matches_native() {
    run_case("single-raft-control", false, 2, false);
}

#[test]
fn per_object_skirt_support_red_matches_native() {
    run_case("support", true, 0, true);
}

#[test]
fn per_object_skirt_raft_red_matches_native() {
    run_case("raft", true, 2, true);
}

fn run_case(label: &str, enable_support: bool, raft_layers: u32, support_fixture: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let Some(resources) =
        crate::native_environment::value("ARES_ORCA_RESOURCES_DIR").map(std::path::PathBuf::from)
    else {
        return;
    };
    let profiles =
        crate::presets::VendorProfiles::load(&resources.join("profiles"), "Creality").unwrap();
    let selection = select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let cube = runner::repo_root().join("tests/parity/cube10.stl");
    let overhang = runner::repo_root().join("tests/parity/overhang.stl");
    let mut process = profiles.process(&selection.process).unwrap();
    normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("skirt_type", json!("perobject")),
        ("skirt_loops", json!("2")),
        ("skirt_height", json!("2")),
        ("skirt_distance", json!("6")),
        ("min_skirt_length", json!("0")),
        ("draft_shield", json!("disabled")),
        (
            "enable_support",
            json!(if enable_support { "1" } else { "0" }),
        ),
        ("raft_layers", json!(raft_layers.to_string())),
        ("brim_type", json!("no_brim")),
        ("combine_brims", json!("0")),
        ("enable_prime_tower", json!("0")),
        ("print_sequence", json!("by layer")),
    ] {
        process.insert(key.to_owned(), value);
    }
    if enable_support {
        process.insert("support_type".into(), json!("normal(auto)"));
        process.insert("support_threshold_angle".into(), json!("45"));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let model = if support_fixture { &overhang } else { &cube };
    let model_list = if label == "single-raft-control" {
        vec![model.as_path()]
    } else {
        vec![model.as_path(), model.as_path()]
    };
    let exported = runner
        .export_case_multi(
            &CaseInputs {
                label: &format!("per-object-skirt-support-raft/{label}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &model_list,
            &[],
        )
        .unwrap();
    let project = if label == "single-raft-control" {
        exported.project
    } else {
        spread_second_object(&exported.project)
    };
    compare_case(&runner, label, &project);
}

fn spread_second_object(project: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "3D/3dmodel.model" {
            let mut xml = String::from_utf8(bytes).unwrap();
            let build_start = xml.find("<build").unwrap();
            let build_end = build_start + xml[build_start..].find("</build>").unwrap();
            let build = &xml[build_start..build_end];
            let item_start = build.match_indices("<item ").nth(1).unwrap().0;
            let transform_start = item_start + build[item_start..].find("transform=\"").unwrap();
            let value_start = transform_start + "transform=\"".len();
            let value_end = value_start + build[value_start..].find('"').unwrap();
            let mut values = build[value_start..value_end]
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            assert_eq!(values.len(), 12);
            values[10] = "170".to_owned();
            xml.replace_range(
                build_start + value_start..build_start + value_end,
                &values.join(" "),
            );
            bytes = xml.into_bytes();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn compare_case(runner: &OrcaRunner, label: &str, project: &[u8]) {
    let root = tempfile::Builder::new()
        .prefix(&format!("per-object-skirt-support-raft-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("input.3mf");
    let native_dir = root.join("native");
    fs::write(&input, project).unwrap();
    fs::create_dir(&native_dir).unwrap();
    runner::stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            native_dir.to_str().unwrap().to_owned(),
            input.to_str().unwrap().to_owned(),
        ],
        runner::stages::Stage::Slice,
        &root,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    fs::write(root.join("native.gcode"), &native).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 10, 17, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.api.gcode"), gcode).unwrap();
    }
    let api_result = api
        .as_ref()
        .map_err(|error| format!("API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&native, gcode));
    let cli_path = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&cli_path);
    let cli_result = cli_bytes
        .as_ref()
        .map_err(|error| error.to_string())
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&native, gcode));
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case": label,
            "native_sha256": sha(&native),
            "api_error": api_result.as_ref().err(),
            "cli_code": cli.status.code(),
            "cli_error": cli_result.as_ref().err(),
            "strict_parity_required": true,
            "upstream": "Print.cpp:2669-2696; GCode.cpp:4311-4318,4398,4455-4485,5329-5335"
        }))
        .unwrap(),
    )
    .unwrap();
    api_result.unwrap_or_else(|error| panic!("{label}: API RED at {root:?}: {error}"));
    assert!(cli.status.success(), "{label}: CLI RED at {root:?}");
    cli_result.unwrap_or_else(|error| panic!("{label}: CLI RED at {root:?}: {error}"));
    assert_eq!(
        api.unwrap(),
        cli_bytes.unwrap(),
        "{label}: API/CLI raw equality"
    );
}

fn sha(bytes: &[u8]) -> String {
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
