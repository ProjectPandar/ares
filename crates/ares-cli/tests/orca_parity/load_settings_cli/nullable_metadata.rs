use std::fs;

use serde_json::{Value, json};

use crate::{golden, native_wall_clock::result_comparison_bytes, runner::stages};

use super::preset_files;

const CASES: &[(&str, &str, &str)] = &[
    ("nil", "bridge_speed", "nil"),
    ("leading-nil", "bridge_speed", "nil,50"),
    ("trailing-nil", "bridge_speed", "50,nil"),
    ("finite-control", "bridge_speed", "50,75"),
    ("support-nil", "small_support_perimeter_speed", "nil"),
    (
        "support-leading-nil",
        "small_support_perimeter_speed",
        "nil,50%",
    ),
];

#[tokio::test]
async fn load_settings_nullable_float_vectors_match_native() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };

    for &(label, key, value) in CASES {
        run_case(&runner, label, key, value);
    }
}

fn run_case(runner: &crate::runner::OrcaRunner, label: &str, key: &str, value: &str) {
    let directory = tempfile::Builder::new()
        .prefix(&format!("load-settings-nullable-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&directory, "Creality", "Creality Ender-3 0.4 nozzle");
    let mut process: Value = serde_json::from_slice(&fs::read(&presets[1]).unwrap()).unwrap();
    process[key] = json!(value);
    fs::write(&presets[1], serde_json::to_vec_pretty(&process).unwrap()).unwrap();
    fs::write(
        directory.join("cube10.stl"),
        fs::read(
            crate::runner::repo_root()
                .join("tests/parity/counterbore-partiallybridge/solid-plate.stl"),
        )
        .unwrap(),
    )
    .unwrap();

    let machine_process = format!("{};{}", presets[0], presets[1]);
    let filaments = presets[2..].join(";");
    let native_output = directory.join("native-output");
    fs::create_dir(&native_output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--load-settings".to_owned(),
            machine_process.clone(),
            "--load-filaments".to_owned(),
            filaments.clone(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            native_output.to_str().unwrap().to_owned(),
            directory.join("cube10.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();

    let command = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("command-")
        })
        .unwrap();
    let ares_output = directory.join("ares-output");
    fs::create_dir(&ares_output).unwrap();
    let ares = crate::runner::application::cli_command()
        .current_dir(&directory)
        .args([
            "slice",
            "--load-settings",
            &machine_process,
            "--load-filaments",
            &filaments,
            "-o",
        ])
        .arg(ares_output.join("plate_1.gcode"))
        .arg(directory.join("cube10.stl"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &ares.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &ares.stderr).unwrap();

    assert_eq!(
        ares.status.code(),
        Some(0),
        "{label}: Ares failed; stderr {}; artifacts {}",
        String::from_utf8_lossy(&ares.stderr),
        directory.display()
    );
    let native_stdout = fs::read(command.join("stdout.bin")).unwrap();
    let native_stderr = fs::read(command.join("stderr.bin")).unwrap();
    assert_eq!(ares.stdout, native_stdout, "{label}: stdout");
    assert_eq!(ares.stderr, native_stderr, "{label}: stderr");
    assert!(
        !native_stderr
            .windows(b"config_value_substituted".len())
            .any(|window| { window == b"config_value_substituted" })
    );
    assert!(
        !ares
            .stderr
            .windows(b"config_value_substituted".len())
            .any(|window| { window == b"config_value_substituted" })
    );

    let native = fs::read(native_output.join("plate_1.gcode")).unwrap();
    let actual = fs::read(ares_output.join("plate_1.gcode")).unwrap();
    let native =
        golden::normalize_one_generator_line(&native, golden::GeneratorKind::Orca).unwrap();
    let actual =
        golden::normalize_one_generator_line(&actual, golden::GeneratorKind::Ares).unwrap();
    assert!(
        native == actual,
        "{label}: complete G-code differs; artifacts {}",
        directory.display()
    );
    let native_result = fs::read(native_output.join("result.json")).unwrap();
    let ares_result = fs::read(ares_output.join("result.json")).unwrap();
    assert!(
        !native_result
            .windows(b"config_value_substituted".len())
            .any(|window| { window == b"config_value_substituted" })
    );
    assert!(
        !ares_result
            .windows(b"config_value_substituted".len())
            .any(|window| { window == b"config_value_substituted" })
    );
    assert_eq!(
        result_comparison_bytes(&native_result),
        result_comparison_bytes(&ares_result),
        "{label}: complete result.json; artifacts {}",
        directory.display()
    );
}
