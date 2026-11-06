//! ToolOrdering.cpp:1150-1206; FilamentGroup.cpp:40-50; stock TPU is right-only.
//! Failure inventory: overwritten mode/map, wrong TPU trigger, wrong base index,
//! missing rejection, CLI channel mismatch, and footer-only false positive.
#![cfg(target_os = "linux")]

#[path = "orca_parity/runner/application/cli_command.rs"]
mod cli_command;
#[path = "stock_project_whole_file/golden.rs"]
mod golden;
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;
#[path = "orca_parity/native_wall_clock.rs"]
mod native_wall_clock;
#[path = "support/project_preset_catalog.rs"]
mod project_presets;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

const INPUT: &[u8] =
    include_bytes!("../../../tests/parity/master-extruder-id/stock-right-only.3mf");
const MESSAGE: &str = "Grouping error: TPU can not be placed in the left nozzle";

fn project(master: &str) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(INPUT)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let before: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(before["filament_type"], json!(["TPU", "PLA"]));
            assert_eq!(before["filament_printable"], json!(["2", "3"]));
            assert_eq!(
                before["nozzle_volume_type"],
                json!(["Standard", "Standard"])
            );
            assert_eq!(before["filament_map_mode"], "Auto For Flush");
            let text = String::from_utf8(bytes).unwrap();
            let needle = "\"master_extruder_id\": \"1\"";
            assert_eq!(text.matches(needle).count(), 1);
            bytes = text
                .replacen(needle, &format!("\"master_extruder_id\": \"{master}\""), 1)
                .into_bytes();
            let after: Value = serde_json::from_slice(&bytes).unwrap();
            let mut expected = before;
            expected["master_extruder_id"] = json!(master);
            assert_eq!(after, expected);
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn native_case(master: &str) -> Option<(PathBuf, Vec<u8>, Output)> {
    let bin = native_environment::binary("ARES_ORCA_BIN")?;
    let native = native_environment::binary("ARES_ORCA_NATIVE_BIN")?;
    let root = std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&root).unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("master-extruder-{master}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    let project = project(master);
    let input = directory.join("input.3mf");
    fs::write(&input, &project).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    fs::write(
        directory.join("native.launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":bin,"native_binary":native,
            "native_binary_sha256":sha(&fs::read(&native).unwrap()),
            "input_sha256":sha(&project),"master":master,
            "args":["--slice","0","--outputdir",output_dir.to_str().unwrap(),input.to_str().unwrap()],
            "seed":std::env::var("ORCA_REPLAY_SEED").ok(),
            "cpu":std::env::var("ORCA_REPLAY_CPU").ok(),
            "APPDIR":std::env::var("ORCA_APPDIR").ok()
        }))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(bin)
        .args(["--slice", "0", "--outputdir"])
        .arg(&output_dir)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(directory.join("native.stdout"), &output.stdout).unwrap();
    fs::write(directory.join("native.stderr"), &output.stderr).unwrap();
    fs::write(
        directory.join("native.exit.json"),
        serde_json::to_vec_pretty(&json!({"code":output.status.code()})).unwrap(),
    )
    .unwrap();
    Some((directory, project, output))
}

fn ares_case(
    directory: &Path,
    project: &[u8],
) -> (Result<Vec<u8>, ares_core::SliceError>, Output, PathBuf) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(project_presets::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    let observation = match &public {
        Ok(gcode) => {
            fs::write(directory.join("ares.validated.gcode"), gcode).unwrap();
            json!({"accepted_bytes":gcode.len(),"sha256":sha(gcode)})
        }
        Err(error) => json!({"error":format!("{error:?}"),"display":error.to_string()}),
    };
    let output_dir = directory.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let output = output_dir.join("plate_1.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = cli_command::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    fs::write(
        directory.join("ares.observation.json"),
        serde_json::to_vec_pretty(&json!({
            "public":observation,"CLI_code":cli.status.code(),
            "CLI_binary":binary,"CLI_binary_sha256":sha(&fs::read(binary).unwrap()),
            "output":output,"result_present":output_dir.join("result.json").is_file()
        }))
        .unwrap(),
    )
    .unwrap();
    (public, cli, output)
}

#[test]
fn master_extruder_stock_right_only_rejects_left_validated_api_and_cli() {
    let Some((directory, project, native)) = native_case("1") else {
        return;
    };
    assert_eq!(native.status.code(), Some(156), "{directory:?}");
    assert_eq!(native.stderr, format!("{MESSAGE}\n").as_bytes());
    let result = fs::read(directory.join("native/result.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&result).unwrap()["return_code"],
        -100
    );
    assert!(!directory.join("native/plate_1.gcode").exists());
    let (public, cli, output) = ares_case(&directory, &project);
    assert_eq!(cli.status.code(), native.status.code(), "{directory:?}");
    assert_eq!(cli.stdout, native.stdout, "{directory:?}");
    assert_eq!(cli.stderr, native.stderr, "{directory:?}");
    assert_eq!(public.unwrap_err().to_string(), MESSAGE, "{directory:?}");
    assert_eq!(
        native_wall_clock::result_comparison_bytes(
            &fs::read(directory.join("ares/result.json")).unwrap()
        ),
        native_wall_clock::result_comparison_bytes(&result),
        "{directory:?}"
    );
    assert!(!output.exists(), "{directory:?}");
}

#[test]
fn master_extruder_stock_right_only_maps_tpu_right_whole_output() {
    let Some((directory, project, native)) = native_case("2") else {
        return;
    };
    assert_eq!(native.status.code(), Some(0), "{directory:?}");
    let reference = fs::read(directory.join("native/plate_1.gcode")).unwrap();
    let text = std::str::from_utf8(&reference).unwrap();
    for witness in [
        "; filament_map = 2,1\n",
        "; filament_printable = 2,3\n",
        "; G4_ACTIVE F0 N2\n",
        "; G4_ACTIVE F1 N1\n",
        "; FEATURE: Sparse infill\n",
    ] {
        assert!(text.contains(witness), "missing {witness:?}: {directory:?}");
    }
    let (public, cli, output) = ares_case(&directory, &project);
    let comparison = public
        .as_ref()
        .map_err(|error| format!("validated API rejected: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, gcode));
    fs::write(
        directory.join("ares.validated.compare.txt"),
        format!("{comparison:?}\n"),
    )
    .unwrap();
    if output.exists() {
        let comparison =
            golden::compare_ordered_bytes_generator_only(&reference, &fs::read(&output).unwrap());
        fs::write(
            directory.join("ares.cli.compare.txt"),
            format!("{comparison:?}\n"),
        )
        .unwrap();
    }
    assert!(cli.status.success(), "{directory:?}: {cli:?}");
    comparison.unwrap_or_else(|error| panic!("{directory:?}: {error}"));
    golden::compare_ordered_bytes_generator_only(&reference, &fs::read(output).unwrap())
        .unwrap_or_else(|error| panic!("{directory:?}: {error}"));
}
