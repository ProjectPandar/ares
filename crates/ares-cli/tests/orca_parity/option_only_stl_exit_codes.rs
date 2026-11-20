use std::{fs, path::Path, process::Command};

use crate::{
    artifacts,
    runner::{
        OrcaRunner,
        stages::{self, FailureKind, Stage},
    },
};

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/cube10.stl");
const MISSING_RESET: &str = "Relative extruder addressing requires resetting the extruder position at each layer to prevent loss of floating point accuracy. Add \"G92 E0\" to layer_gcode.";
const MODEL_LOAD_FAILED: &str = "Loading of a model file failed.";

#[test]
fn cli_option_only_stl_missing_reset_returns_validate_error() {
    compare_rejection(CUBE, "{}", &[], -51, MISSING_RESET);
}

#[test]
fn cli_option_only_stl_truncated_binary_returns_data_file_error() {
    let mut stl = [0_u8; 80 + 4 + 49];
    stl[80..84].copy_from_slice(&1_u32.to_le_bytes());
    compare_rejection(
        &stl,
        r#"{"before_layer_change_gcode":"G92 E0"}"#,
        &["--before-layer-change-gcode", "G92 E0"],
        -6,
        MODEL_LOAD_FAILED,
    );
}

fn compare_rejection(
    stl: &[u8],
    options: &str,
    native_options: &[&str],
    return_code: i32,
    message: &str,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("option-only-stl-rejection-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("input.stl");
    let options_path = root.join("options.json");
    fs::write(&input, stl).unwrap();
    fs::write(&options_path, options).unwrap();
    let expected_code = if cfg!(unix) {
        return_code.rem_euclid(256)
    } else {
        return_code
    };
    let expected = if message == MODEL_LOAD_FAILED {
        format!("<input>: {message}")
    } else {
        message.to_owned()
    };
    let native = root.join("native");
    fs::create_dir(&native).unwrap();
    let args = ["--slice", "0", "--outputdir", native.to_str().unwrap()]
        .into_iter()
        .chain(native_options.iter().copied())
        .chain([input.to_str().unwrap()])
        .map(str::to_owned);
    let error = stages::run_live(runner.bin(), args, Stage::Slice, &native).unwrap_err();
    assert_eq!(
        error.kind,
        FailureKind::Process,
        "Native at {root:?}: {error}"
    );
    let native_result = error.captured.unwrap();
    assert_eq!(
        native_result.code,
        Some(expected_code),
        "Native at {root:?}"
    );
    let native_key = key_diagnostics(&native_result.stderr, &input, message);
    assert_eq!(native_key, [expected], "Native at {root:?}");
    assert!(
        fs::read_dir(&native).unwrap().all(|entry| {
            entry
                .unwrap()
                .path()
                .extension()
                .is_none_or(|ext| ext != "gcode")
        }),
        "Native wrote G-code at {root:?}"
    );

    let cli_path = root.join("cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--options"])
        .arg(options_path)
        .arg("-o")
        .arg(&cli_path)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    fs::write(root.join("cli.exit"), format!("{:?}\n", cli.status.code())).unwrap();
    assert_eq!(cli.status.code(), native_result.code, "Ares at {root:?}");
    assert_eq!(
        key_diagnostics(&cli.stderr, &input, message),
        native_key,
        "Ares at {root:?}"
    );
    assert!(!cli_path.exists(), "Ares wrote G-code at {root:?}");
}

fn key_diagnostics(stderr: &[u8], input: &Path, message: &str) -> Vec<String> {
    let path_prefix = format!("{}: ", input.display());
    String::from_utf8_lossy(stderr)
        .lines()
        .filter(|line| line.contains(message))
        .map(|line| match line.strip_prefix(&path_prefix) {
            Some(detail) => format!("<input>: {detail}"),
            None => line.to_owned(),
        })
        .collect()
}
