#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::{cli_retained_input::normalize_generator, native_wall_clock::result_comparison_bytes};

fn compare(
    runner: &crate::runner::OrcaRunner,
    case: &str,
    flag: Option<&str>,
    status: i32,
) -> Option<String> {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/allow-mix-temp")
        .join(format!("{case}.3mf"));
    let prefix = format!("cli-allow-mix-temp-{case}-");
    let builder = &mut tempfile::Builder::new();
    builder.prefix(&prefix);
    let root = match std::env::var_os("ARES_PARITY_ARTIFACT_ROOT") {
        Some(root) => builder.tempdir_in(root),
        None => builder.tempdir(),
    }
    .unwrap()
    .keep();
    let native = root.join("native");
    let ares = root.join("ares");
    fs::create_dir(&native).unwrap();
    fs::create_dir(&ares).unwrap();
    let native_output = Command::new(runner.bin())
        .current_dir(&native)
        .args(["--slice", "0", "--outputdir", "."])
        .args(flag)
        .arg(&input)
        .output()
        .unwrap();
    let ares_output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(&ares)
        .args(["slice", "-o", "plate_1.gcode"])
        .args(flag)
        .arg(&input)
        .output()
        .unwrap();
    for (directory, output) in [(&native, &native_output), (&ares, &ares_output)] {
        fs::write(directory.join("stdout"), &output.stdout).unwrap();
        fs::write(directory.join("stderr"), &output.stderr).unwrap();
        fs::write(
            directory.join("exit"),
            format!("{:?}\n", output.status.code()),
        )
        .unwrap();
    }
    assert_eq!(native_output.status.code(), Some(status), "{root:?}");
    assert_eq!(
        ares_output.status.code(),
        native_output.status.code(),
        "{root:?}"
    );
    assert_eq!(ares_output.stderr, native_output.stderr, "{root:?}");
    let result = |directory: &Path| {
        result_comparison_bytes(&fs::read(directory.join("result.json")).unwrap())
    };
    assert_eq!(result(&ares), result(&native), "{root:?}");
    let read = |directory: &Path, producer| {
        normalize_generator(
            &fs::read_to_string(directory.join("plate_1.gcode")).unwrap(),
            producer,
        )
    };
    if status != 0 {
        assert!(!native.join("plate_1.gcode").exists() && !ares.join("plate_1.gcode").exists());
        return None;
    }
    let gcode = read(&ares, "Ares");
    assert!(
        gcode == read(&native, "OrcaSlicer"),
        "G-code differs: {root:?}"
    );
    Some(gcode)
}

#[test]
fn mixed_temperatures_reject_the_plate_by_default() {
    if let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() {
        compare(&runner, "mixed", None, 194);
        compare(&runner, "mixed", Some("--allow-mix-temp=0"), 194);
    }
}

#[test]
fn allowing_mixed_temperatures_preserves_native_gcode() {
    if let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() {
        compare(&runner, "mixed", Some("--allow-mix-temp=1"), 0);
    }
}

#[test]
fn compatible_temperatures_slice_the_same_with_or_without_allowance() {
    if let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() {
        let default = compare(&runner, "compatible", None, 0);
        let enabled = compare(&runner, "compatible", Some("--allow-mix-temp"), 0);
        assert!(default == enabled);
    }
}

#[test]
fn allowing_mixed_temperatures_does_not_allow_an_invalid_range() {
    if let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() {
        compare(&runner, "invalid-range", Some("--allow-mix-temp=1"), 194);
    }
}
