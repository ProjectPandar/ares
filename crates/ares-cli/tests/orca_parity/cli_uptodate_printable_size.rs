//! Native retained-machine printable-size admission (`OrcaSlicer.cpp:2563-2600`).
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use super::cli_uptodate_refresh::{ares, artifact_dir, native_gcode, project};
use crate::runner::OrcaRunner;

const MACHINE: &str = "Prusa MK3S 0.4 nozzle";
const SETTINGS: [&str; 4] = [
    "--uptodate",
    "--uptodate-settings",
    "machine.json;process.json",
    "--uptodate-filaments",
];

#[test]
fn refresh_printable_size_rejects_embedded_dimensions_larger_than_machine() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = &artifact_dir();
    project(root);
    machine_size(root, ["0x0", "249x0", "249x209", "0x209"], "209");
    let args = [SETTINGS.as_slice(), ["filament.json"].as_slice()].concat();
    let native = native_failure(&runner, root, &args);
    let ares = ares((root, "ares"), &args);
    compare_failure(
        root,
        native,
        ares,
        "Printer Settings: the printable size {250, 210, 210} exceeds the default size.",
    );
}

#[test]
fn refresh_printable_size_accepts_exact_embedded_dimensions() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = &artifact_dir();
    project(root);
    machine_size(root, ["0x0", "250x0", "250x210", "0x210"], "210");
    let args = SETTINGS
        .into_iter()
        .chain(["filament.json"])
        .collect::<Vec<_>>();
    let native = native_gcode((&runner, root), "native", &args);
    let output = ares((root, "ares"), &args);
    assert!(
        output.status.success(),
        "equal printable size rejected: {}; {root:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = fs::read(root.join("ares/plate_1.gcode")).unwrap();
    assert_eq!(
        crate::load_settings_cli::banner_normalized(&native),
        crate::load_settings_cli::banner_normalized(&actual),
        "equal printable size changed complete G-code; {root:?}"
    );
}

fn machine_size(root: &Path, area: [&str; 4], height: &str) {
    let path = root.join("machine.json");
    let mut machine: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    machine["printable_area"] = json!(area);
    machine["printable_height"] = json!(height);
    fs::write(path, serde_json::to_vec(&machine).unwrap()).unwrap();
}

fn native_failure(runner: &OrcaRunner, root: &Path, args: &[&str]) -> std::process::Output {
    fs::create_dir(root.join("native")).unwrap();
    Command::new(runner.bin())
        .current_dir(root)
        .args(args)
        .args(["--slice", "0", "--outputdir", "native", "retained.3mf"])
        .output()
        .unwrap()
}

fn compare_failure(
    root: &Path,
    native: std::process::Output,
    ares: std::process::Output,
    text: &str,
) {
    for (dir, output) in [("native", native), ("ares", ares)] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(237), "{dir}: {stderr}; {root:?}");
        assert!(
            stderr.contains(text),
            "{dir} missing {text}: {stderr}; {root:?}"
        );
        let report: Value =
            serde_json::from_slice(&fs::read(root.join(dir).join("result.json")).unwrap()).unwrap();
        assert_eq!(report["return_code"], json!(-19), "{dir}; {root:?}");
        assert_eq!(report["error_string"], json!(text), "{dir}; {root:?}");
        assert!(!root.join(dir).join("plate_1.gcode").exists());
    }
}
