//! WipeTower2.cpp:1380-1390,1643-1657,1707-1713,1834-1844.
//! Failure inventory: profile collapse, inactive tower, wrong gate/filament,
//! header-only activation, off XY assumption, API/CLI whole-output mismatch.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};
#[path = "semm_ramming/input.rs"]
mod input;
use input::{DEFAULT, SHORT, WIDTH};

struct Case {
    label: &'static str,
    profiles: [&'static str; 2],
    gate: &'static str,
}
const CASES: [Case; 7] = [
    Case {
        label: "R0",
        profiles: [DEFAULT, DEFAULT],
        gate: "1",
    },
    Case {
        label: "R1",
        profiles: [DEFAULT, SHORT],
        gate: "1",
    },
    Case {
        label: "R2",
        profiles: [SHORT, DEFAULT],
        gate: "1",
    },
    Case {
        label: "R3",
        profiles: [SHORT, SHORT],
        gate: "1",
    },
    Case {
        label: "R4",
        profiles: [DEFAULT, WIDTH],
        gate: "1",
    },
    Case {
        label: "C1",
        profiles: [DEFAULT, SHORT],
        gate: "0",
    },
    Case {
        label: "C2",
        profiles: [SHORT, DEFAULT],
        gate: "0",
    },
];

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

fn extrusion_lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| (line.starts_with("G0 ") || line.starts_with("G1 ")) && line.contains(" E"))
        .collect()
}

fn assert_unload(native: &[u8], active: bool) -> usize {
    let text = std::str::from_utf8(native).unwrap();
    let mut count = 0;
    for tail in text.split("; CP TOOLCHANGE UNLOAD\n").skip(1) {
        let block = tail.split_once("; filament end gcode \n").unwrap().0;
        assert_eq!(block.contains("; Ramming start\n"), active);
        assert_eq!(block.contains("; Ramming end\n"), active);
        if !active {
            // Source1707 do_ramming gates positive ramming E moves. Stock unload
            // retractions remain; later wiping XY still depends on profile y_step.
            for word in block
                .lines()
                .filter(|l| l.starts_with("G1 ") || l.starts_with("G0 "))
                .flat_map(str::split_whitespace)
                .filter_map(|word| word.strip_prefix('E'))
            {
                assert!(
                    word.parse::<f64>().unwrap() <= 0.0,
                    "off unload ramming E: {word}"
                );
            }
        }
        count += 1;
    }
    assert!(count >= 2, "must unload both logical materials");
    assert!(text.matches("\nM600\n").count() >= 2);
    count
}

fn compare(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    compare_with(
        &runner,
        CASES.iter().find(|case| case.label == label).unwrap(),
    );
}

fn compare_with(runner: &OrcaRunner, case: &Case) {
    let root = tempfile::Builder::new()
        .prefix(&format!("semm-ramming-{}-", case.label))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = input::project(&case.profiles, case.gate);
    let reference = native(runner, &directory, &project);
    let active = case.gate == "1";
    let unload_count = assert_unload(&reference, active);
    if matches!(case.label, "R2" | "R4") {
        let baseline = native(
            runner,
            &root.join("baseline"),
            &input::project(&[DEFAULT, SHORT], "1"),
        );
        assert_ne!(
            extrusion_lines(&reference),
            extrusion_lines(&baseline),
            "profile/width must affect Native motion, not only config text"
        );
    }
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
            "label":case.label,"requested_profiles":case.profiles,"enable_filament_ramming":case.gate,
            "input_sha256":sha(&project),"native_sha256":sha(&reference),
            "validated_api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "CLI_code":cli.status.code(),"CLI_binary":binary,
            "CLI_binary_sha256":sha(&fs::read(binary).unwrap()),
            "native_unload_count":unload_count,"ramming_enabled":active,
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
fn semm_ramming_r0_whole_output() {
    compare("R0");
}

#[test]
fn semm_ramming_r1_whole_output() {
    compare("R1");
}

#[test]
fn semm_ramming_r2_whole_output() {
    compare("R2");
}

#[test]
fn semm_ramming_r3_whole_output() {
    compare("R3");
}

#[test]
fn semm_ramming_r4_whole_output() {
    compare("R4");
}

#[test]
fn semm_ramming_c1_whole_output() {
    compare("C1");
}

#[test]
fn semm_ramming_c2_whole_output() {
    compare("C2");
}
