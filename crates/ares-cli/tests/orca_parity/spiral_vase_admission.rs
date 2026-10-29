//! `Print.cpp:1334-1341` spiral-vase admission: with `spiral_mode` on, a
//! by-layer print of more than one copy is rejected before slicing with the
//! upstream diagnostic on the validate channel (`CLI_VALIDATE_ERROR` -51,
//! `Utils.hpp:62`; the `result.json` validate fields of
//! `OrcaSlicer.cpp:5981-5987`). Guards prove no over-rejection: one copy by
//! layer and two separated copies by object stay accepted by Native and
//! Ares — acceptance only, because accepted-scene BBL spiral bytes diverge
//! on this tip (findings recorded in the spec's deferred section).
#![cfg(target_os = "linux")]

use std::{fs, process::Command};

use crate::{native_wall_clock::result_comparison_bytes, runner::EarlyExit};

const WITNESS: &[u8] =
    include_bytes!("../../../../tests/parity/spiral-vase-admission/two-object-by-layer.3mf");
const SINGLE_COPY: &[u8] =
    include_bytes!("../../../../tests/parity/spiral-vase-admission/single-object-by-layer.3mf");
const BY_OBJECT_APART: &[u8] =
    include_bytes!("../../../../tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf");

#[test]
fn spiral_vase_multi_copy_by_layer_rejected_like_native() {
    let root = crate::artifacts::root_from_env().unwrap();
    let artifacts = tempfile::Builder::new()
        .prefix("spiral-vase-witness-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let project = artifacts.join("two-object-by-layer.3mf");
    fs::write(&project, WITNESS).unwrap();
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let native = runner
        .early_cli_reference(&project, EarlyExit::Validation)
        .unwrap();
    let replay = runner
        .early_cli_reference(&project, EarlyExit::Validation)
        .unwrap();
    assert_eq!(native.directory, replay.directory);
    assert_eq!(native.stdout, replay.stdout);
    assert_eq!(native.stderr, replay.stderr);
    assert_eq!(native.result_json, replay.result_json);
    assert!(native.gcode.is_none());
    assert!(
        String::from_utf8_lossy(&native.stderr).contains("spiral vase mode"),
        "Native diagnostic: {:?}",
        String::from_utf8_lossy(&native.stderr)
    );
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        artifacts.join("native.receipt.json"),
    )
    .unwrap();
    // "run found error, return -51, exit...\n"
    let code = String::from_utf8(native.stdout.clone())
        .unwrap()
        .split("return ")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .parse::<i32>()
        .unwrap();
    assert_eq!(code, -51, "{artifacts:?}");

    let gcode = artifacts.join("plate_1.gcode");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&gcode)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    // flush_and_exit returns the negative code; POSIX exposes 256+n.
    assert_eq!(
        output.status.code(),
        Some(256 + code),
        "{artifacts:?}: Ares must reject the multi-copy by-layer spiral print"
    );
    assert_eq!(
        output.stdout, native.stdout,
        "{artifacts:?}: stdout channel"
    );
    assert_eq!(
        output.stderr, native.stderr,
        "{artifacts:?}: stderr channel"
    );
    assert_eq!(
        result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
        result_comparison_bytes(&native.result_json),
        "{artifacts:?}: complete result.json bytes"
    );
    assert!(!gcode.exists(), "rejection must not export G-code");
}

#[test]
fn spiral_vase_single_copy_by_layer_accepted_like_native() {
    accepted_by_both("single-object-by-layer", SINGLE_COPY);
}

#[test]
fn spiral_vase_multi_copy_by_object_accepted_like_native() {
    accepted_by_both("two-object-by-object-apart", BY_OBJECT_APART);
}

/// Guard: Native slices the project (exit 0 with `plate_1.gcode`) and Ares
/// keeps accepting it — the admission check must not over-reject.
fn accepted_by_both(name: &str, project_bytes: &[u8]) {
    let root = crate::artifacts::root_from_env().unwrap();
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("spiral-vase-{name}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    let project = artifacts.join(format!("{name}.3mf"));
    fs::write(&project, project_bytes).unwrap();
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let output = artifacts.join("native");
    fs::create_dir(&output).unwrap();
    crate::runner::stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            project.to_str().unwrap().to_owned(),
        ],
        crate::runner::stages::Stage::Slice,
        &artifacts,
    )
    .unwrap();
    assert!(
        fs::read(output.join("plate_1.gcode")).is_ok(),
        "{name}: Native must accept: {artifacts:?}"
    );
    let gcode = artifacts.join("ares.gcode");
    let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&gcode)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("ares.stdout"), &ares.stdout).unwrap();
    fs::write(artifacts.join("ares.stderr"), &ares.stderr).unwrap();
    fs::write(
        artifacts.join("ares.exit"),
        format!("{:?}\n", ares.status.code()),
    )
    .unwrap();
    assert_eq!(
        ares.status.code(),
        Some(0),
        "{name}: Ares must keep accepting: {artifacts:?}"
    );
    assert!(gcode.is_file(), "{name}: {artifacts:?}");
}
