use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

const PRINTER: &str = "Prusa CORE One 0.4 nozzle";
const REJECTION: &str =
    "The selected printer is not compatible with the process preset in the 3mf.";
const FALSE_CONDITION: &str = "nozzle_diameter[0]==0.6";

#[test]
fn stock_condition_accepts_the_printer() {
    compare("stock-condition", |_| {}, 0);
}

#[test]
fn empty_list_with_no_condition_is_unrestricted() {
    compare(
        "unrestricted",
        |preset| {
            preset
                .as_object_mut()
                .unwrap()
                .remove("compatible_printers_condition");
        },
        0,
    );
}

#[test]
fn absent_list_with_false_condition_rejects_the_printer() {
    compare(
        "false-condition",
        |preset| {
            preset
                .as_object_mut()
                .unwrap()
                .remove("compatible_printers");
            preset["compatible_printers_condition"] = json!(FALSE_CONDITION);
        },
        -17,
    );
}

#[test]
fn literal_match_takes_precedence_over_false_condition() {
    compare(
        "literal-match",
        |preset| {
            preset["compatible_printers"] = json!([PRINTER]);
            preset["compatible_printers_condition"] = json!(FALSE_CONDITION);
        },
        0,
    );
}

#[test]
fn literal_miss_takes_precedence_over_true_condition() {
    compare(
        "literal-miss",
        |preset| {
            preset["compatible_printers"] = json!(["Prusa CORE One 0.6 nozzle"]);
        },
        -17,
    );
}

fn compare(label: &str, edit_process: impl FnOnce(&mut Value), expected_code: i32) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("direct-stl-compatibility-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    eprintln!(
        "direct STL compatibility {label}: artifacts {}",
        root.display()
    );
    let fixture = crate::runner::repo_root().join("tests/parity/direct-stl-process-compatibility");
    let mut process: Value =
        serde_json::from_slice(&fs::read(fixture.join("process.json")).unwrap()).unwrap();
    edit_process(&mut process);
    for name in ["cube10.stl", "machine.json", "filament.json"] {
        fs::copy(fixture.join(name), root.join(name)).unwrap();
    }
    fs::write(
        root.join("process.json"),
        serde_json::to_vec_pretty(&process).unwrap(),
    )
    .unwrap();

    let native_output = root.join("native");
    let ares_output = root.join("ares");
    fs::create_dir(&native_output).unwrap();
    fs::create_dir(&ares_output).unwrap();
    let settings = ["machine.json", "process.json"]
        .map(|name| root.join(name).display().to_string())
        .join(";");
    let filaments = root.join("filament.json");
    let stl = root.join("cube10.stl");
    let shared = [
        "--load-settings",
        &settings,
        "--load-filaments",
        filaments.to_str().unwrap(),
        stl.to_str().unwrap(),
    ];

    let native_exit = run(
        &root,
        "native",
        Command::new(runner.bin())
            .args([
                "--slice",
                "0",
                "--arrange",
                "0",
                "--outputdir",
                native_output.to_str().unwrap(),
            ])
            .args(shared),
    );
    let expected_exit = expected_code.rem_euclid(256);
    assert_eq!(native_exit, Some(expected_exit), "Native exit: {root:?}");
    let native_report = report(&native_output);
    assert_eq!(
        native_report["return_code"], expected_code,
        "Native report: {root:?}"
    );

    let ares_file = ares_output.join("plate_1.gcode");
    let ares_exit = run(
        &root,
        "ares",
        Command::new(env!("CARGO_BIN_EXE_ares"))
            .arg("slice")
            .arg("-o")
            .arg(&ares_file)
            .args(shared),
    );
    let ares_report = report(&ares_output);
    assert_eq!(
        ares_exit, native_exit,
        "exit: {root:?}; Ares report: {ares_report}"
    );
    assert_eq!(
        ares_report["return_code"], native_report["return_code"],
        "return code: {root:?}"
    );
    let native_file = native_output.join("plate_1.gcode");
    if expected_code != 0 {
        assert_eq!(native_report["error_string"], REJECTION, "{root:?}");
        assert_eq!(
            ares_report["error_string"], native_report["error_string"],
            "{root:?}"
        );
        assert!(
            !native_file.exists() && !ares_file.exists(),
            "unexpected G-code: {root:?}"
        );
        return;
    }
    let native_gcode = fs::read(&native_file).unwrap();
    let ares_gcode = fs::read(&ares_file).unwrap();
    assert!(
        !native_gcode.is_empty() && !ares_gcode.is_empty(),
        "{root:?}"
    );
    assert!(
        super::banner_normalized(&native_gcode) == super::banner_normalized(&ares_gcode),
        "whole G-code divergence: {root:?}"
    );
}

fn report(directory: &Path) -> Value {
    let path = directory.join("result.json");
    serde_json::from_slice(&fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}")))
        .unwrap()
}

fn run(root: &Path, side: &str, command: &mut Command) -> Option<i32> {
    let argv: Vec<_> = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|part| part.to_str().unwrap())
        .collect();
    fs::write(
        root.join(format!("{side}.argv.json")),
        serde_json::to_vec_pretty(&argv).unwrap(),
    )
    .unwrap();
    let output = command.current_dir(root).output().unwrap();
    fs::write(root.join(format!("{side}.stdout")), output.stdout).unwrap();
    fs::write(root.join(format!("{side}.stderr")), output.stderr).unwrap();
    fs::write(
        root.join(format!("{side}.exit")),
        format!("{}\n", output.status),
    )
    .unwrap();
    output.status.code()
}
