//! FlushVolPredictor.cpp:312-327; FlushVolCalc.cpp:43-124.
//! Failure inventory: collapsed dataset, wrong selected variant, stale matrix,
//! rebuilding supplied matrix, multiplier applied twice, API/CLI mismatch.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
    process::Command,
};
struct Case {
    label: &'static str,
    dataset: [u8; 4],
    matrix: &'static str,
    explicit: bool,
    project: &'static [u8],
}
const CASES: [Case; 6] = [
    Case {
        label: "D0",
        dataset: [0, 0, 0, 0],
        matrix: "0,98,46,0,0,328,152,0",
        explicit: false,
        project: include_bytes!("../../../../../tests/parity/nozzle-flush-dataset/D0.3mf"),
    },
    Case {
        label: "D1",
        dataset: [1, 1, 1, 1],
        matrix: "0,135,27,0,0,450,90,0",
        explicit: false,
        project: include_bytes!("../../../../../tests/parity/nozzle-flush-dataset/D1.3mf"),
    },
    Case {
        label: "D2",
        dataset: [2, 2, 2, 2],
        matrix: "0,81,45,0,0,270,150,0",
        explicit: false,
        project: include_bytes!("../../../../../tests/parity/nozzle-flush-dataset/D2.3mf"),
    },
    Case {
        label: "DM",
        dataset: [1, 2, 2, 1],
        matrix: "0,135,27,0,0,270,150,0",
        explicit: false,
        project: include_bytes!("../../../../../tests/parity/nozzle-flush-dataset/DM.3mf"),
    },
    Case {
        label: "C1",
        dataset: [1, 1, 1, 1],
        matrix: "0,42,42,0,0,210,210,0",
        explicit: true,
        project: include_bytes!("../../../../../tests/parity/nozzle-flush-dataset/C1.3mf"),
    },
    Case {
        label: "C2",
        dataset: [2, 2, 2, 2],
        matrix: "0,42,42,0,0,210,210,0",
        explicit: true,
        project: include_bytes!("../../../../../tests/parity/nozzle-flush-dataset/C2.3mf"),
    },
];
fn verify_input(case: &Case) {
    let mut zip = zip::ZipArchive::new(Cursor::new(case.project)).unwrap();
    let mut bytes = Vec::new();
    zip.by_name("Metadata/project_settings.config")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        settings["nozzle_flush_dataset"],
        json!(case.dataset.map(|v| v.to_string()))
    );
    assert_eq!(settings["filament_colour"], json!(["#000000", "#545454"]));
    assert_eq!(settings["flush_multiplier"], json!(["0.3", "1"]));
    assert_eq!(settings["filament_map"], json!(["1", "1"]));
    assert_eq!(settings["nozzle_diameter"], json!(["0.4", "0.4"]));
    if case.explicit {
        assert_eq!(
            settings["flush_volumes_matrix"],
            json!(["0", "140", "140", "0", "0", "210", "210", "0"])
        );
    } else {
        assert!(settings.get("flush_volumes_matrix").is_none());
    }
}
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn native(runner: &OrcaRunner, directory: &Path, project: &[u8]) -> Vec<u8> {
    fs::create_dir(directory).unwrap();
    let path = directory.join("input.3mf");
    fs::write(&path, project).unwrap();
    let output = directory.join("native");
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        output.to_str().unwrap().into(),
        path.to_str().unwrap().into(),
    ];
    let producer = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        directory.join("native.launch.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":sha(project),"native_binary":producer,
            "native_sha256":sha(&fs::read(&producer).unwrap()),
            "wrapper":runner.bin(),"args":args,
            "seed":std::env::var("ORCA_REPLAY_SEED").ok(),
            "cpu":std::env::var("ORCA_REPLAY_CPU").ok()
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, directory).unwrap();
    let result = fs::read(output.join("result.json")).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(metadata["return_code"], 0, "{directory:?}");
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(!reference.is_empty());
    reference
}

fn compare(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    compare_with(&runner, CASES.iter().find(|c| c.label == label).unwrap());
}
fn compare_with(runner: &OrcaRunner, case: &Case) {
    verify_input(case);
    let root = tempfile::Builder::new()
        .prefix(&format!("nozzle-dataset-{}-", case.label))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = case.project;
    let reference = native(runner, &directory, project);
    let text = std::str::from_utf8(&reference).unwrap();
    assert!(text.contains(&format!("; flush_volumes_matrix = {}\n", case.matrix)));
    let selected = if case.label == "DM" {
        "1,2"
    } else {
        match case.label {
            "D0" => "0,0",
            "D1" | "C1" => "1,1",
            "D2" | "C2" => "2,2",
            _ => unreachable!(),
        }
    };
    assert!(text.contains(&format!("; nozzle_flush_dataset = {selected}\n")));
    assert!(text.contains("; flush_multiplier = 0.3,1\n"));
    assert!(text.contains("; X2D_FM_BEGIN\n") && text.contains("; X2D_FM_END\n"));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(ares_core::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    if let Ok(gcode) = &public {
        fs::write(directory.join("ares.validated.gcode"), gcode).unwrap();
    }
    let api_comparison = public
        .as_ref()
        .map_err(|error| format!("validated API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, gcode));
    let output_dir = directory.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let output = output_dir.join("plate_1.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    let cli_comparison = fs::read(&output)
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, &gcode));
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":case.label,"requested_dataset":case.dataset,"expected_matrix":case.matrix,
            "input_sha256":sha(project),"native_sha256":sha(&reference),
            "validated_api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "CLI_code":cli.status.code(),"CLI_binary":binary,
            "CLI_binary_sha256":sha(&fs::read(binary).unwrap()),
            "explicit_matrix":case.explicit,
            "api_error":public.as_ref().err().map(|error| format!("{error:?}"))
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}

#[test]
fn nozzle_dataset_d0_whole_output() {
    compare("D0");
}

#[test]
fn nozzle_dataset_d1_whole_output() {
    compare("D1");
}

#[test]
fn nozzle_dataset_d2_whole_output() {
    compare("D2");
}

#[test]
fn nozzle_dataset_dm_whole_output() {
    compare("DM");
}

#[test]
fn nozzle_dataset_c1_whole_output() {
    compare("C1");
}

#[test]
fn nozzle_dataset_c2_whole_output() {
    compare("C2");
}
