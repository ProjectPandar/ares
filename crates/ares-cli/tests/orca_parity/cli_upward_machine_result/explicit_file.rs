//! OrcaSlicer.cpp:2118–2155,2205–2275,2539–2558: explicit machine-file context.
//! Failure modes: printer-only rejection, empty-inherits misrouting, or
//! mistaking CLI selection/compatibility for API admission.
use std::{fs, path::Path, process::Command};

use ares_core::{LoadSettingsFile, UpwardMachineInput, UpwardMachineSource};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::runner::{self, OrcaRunner};

#[derive(Clone, Copy)]
enum Entry {
    UpToDate,
    CompleteList,
}

#[test]
fn stock_up_to_date_file_uses_full_config_reader() {
    exercise("api-stock", false, Entry::UpToDate);
}

#[test]
fn empty_inherits_up_to_date_file_uses_full_config_reader() {
    exercise("api-empty-inherits", true, Entry::UpToDate);
}

#[test]
fn stock_complete_cli_list_matches_native_compatibility_rejection() {
    exercise("cli-stock", false, Entry::CompleteList);
}

#[test]
fn empty_inherits_complete_cli_list_matches_native_compatibility_rejection() {
    exercise("cli-empty-inherits", true, Entry::CompleteList);
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn channels(root: &Path, prefix: &str, output: &std::process::Output) {
    fs::write(root.join(format!("{prefix}.stdout")), &output.stdout).unwrap();
    fs::write(root.join(format!("{prefix}.stderr")), &output.stderr).unwrap();
    fs::write(
        root.join(format!("{prefix}.exit")),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
}

fn exercise(label: &str, empty_inherits: bool, entry: Entry) {
    if OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("explicit-machine-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let repo = runner::repo_root();
    let original =
        fs::read(repo.join("tests/parity/machine-full-resource/stock-machine-full.json")).unwrap();
    assert_eq!(
        digest(&original),
        "2a2e89e53063119290a5b8c185ea01f4e207044dc11f4e78f5a34cd22d162e4b"
    );
    assert!(
        serde_json::from_slice::<Value>(&original)
            .unwrap()
            .get("cooling_filter_enabled")
            .is_some()
    );
    let mut machine = String::from_utf8(original.clone()).unwrap();
    if empty_inherits {
        machine = machine.replacen("{\n", "{\n    \"inherits\": \"\",\n", 1);
        let mut expected: Value = serde_json::from_slice(&original).unwrap();
        expected["inherits"] = json!("");
        assert_eq!(serde_json::from_str::<Value>(&machine).unwrap(), expected);
    } else {
        assert_eq!(machine.as_bytes(), original);
    }
    let machine_path = root.join("machine.json");
    fs::write(&machine_path, &machine).unwrap();
    let input =
        fs::read(repo.join("tests/parity/master-extruder-id/stock-right-only.3mf")).unwrap();
    assert_eq!(
        digest(&input),
        "69d425b79297129c1b3894ee106269d3b33b7f1badcbeed15598dfe774cff09a"
    );
    let input_path = root.join("stock-right-only.3mf");
    fs::write(&input_path, &input).unwrap();
    let validation = repo.join("tests/parity/validation/load-settings-3mf");
    let process_path = validation.join("process.json");
    let filament_path = validation.join("compatible-filament-0.json");
    let complete_list = format!("{};{}", machine_path.display(), process_path.display());
    let mut native_argv = match entry {
        Entry::UpToDate => vec![
            "--uptodate".to_owned(),
            "--uptodate-settings".to_owned(),
            machine_path.display().to_string(),
        ],
        Entry::CompleteList => vec![
            "--load-settings".to_owned(),
            complete_list.clone(),
            "--load-filaments".to_owned(),
            filament_path.display().to_string(),
        ],
    };
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    native_argv.extend([
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        native_dir.display().to_string(),
        input_path.display().to_string(),
    ]);
    let native_bin = std::env::var_os("ARES_ORCA_BIN").unwrap();
    let native = Command::new(&native_bin)
        .args(&native_argv)
        .current_dir(&root)
        .output()
        .unwrap();
    channels(&root, "native", &native);
    let native_report: Value =
        serde_json::from_slice(&fs::read(native_dir.join("result.json")).unwrap()).unwrap();
    let (api, ares_report, ares_exit) = match entry {
        Entry::UpToDate => {
            let file = LoadSettingsFile {
                label: "machine.json",
                json: &machine,
            };
            let api = UpwardMachineSource::from_project(&input)
                .unwrap()
                .unwrap()
                .select(UpwardMachineInput::UpToDateFiles(&[file]))
                .map_err(|error| error.to_string());
            (Some(api), None, None)
        }
        Entry::CompleteList => {
            let ares_dir = root.join("ares");
            fs::create_dir(&ares_dir).unwrap();
            let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
                .args([
                    "slice",
                    "--load-settings",
                    &complete_list,
                    "--load-filaments",
                ])
                .arg(&filament_path)
                .arg("-o")
                .arg(ares_dir.join("plate_1.gcode"))
                .arg(&input_path)
                .current_dir(&root)
                .output()
                .unwrap();
            channels(&root, "ares", &ares);
            let report: Value =
                serde_json::from_slice(&fs::read(ares_dir.join("result.json")).unwrap()).unwrap();
            (None, Some(report), ares.status.code())
        }
    };
    fs::write(root.join("observation.json"), serde_json::to_vec_pretty(&json!({
        "case":label, "input_sha256":digest(&input), "machine_sha256":digest(machine.as_bytes()),
        "empty_inherits":empty_inherits, "machine_path":machine_path,
        "process_sha256":digest(&fs::read(&process_path).unwrap()),
        "filament_sha256":digest(&fs::read(&filament_path).unwrap()),
        "native_bin":native_bin.to_string_lossy(), "native_argv":native_argv,
        "native_exit":native.status.code(), "native_report":native_report,
        "raw_cli":env!("CARGO_BIN_EXE_ares"), "ares_exit":ares_exit, "ares_report":ares_report,
        "api":api,
        "scope":"Explicit projection admission/upward or complete-list compatibility code; not slicing parity"
    })).unwrap()).unwrap();
    eprintln!("explicit_file {label}: artifacts {root:?}");
    match entry {
        Entry::UpToDate => {
            assert_eq!(native.status.code(), Some(0), "{root:?}");
            assert_eq!(native_report["return_code"], 0, "{root:?}");
            let expected = vec!["Bambu Lab H2D Pro 0.4 nozzle".to_owned()];
            assert_eq!(native_report["upward_compatible_machine"], json!(expected));
            assert_eq!(api, Some(Ok(expected)), "UpToDateFiles: {root:?}");
        }
        Entry::CompleteList => {
            assert_eq!(native.status.code(), Some(239), "{root:?}");
            assert_eq!(ares_exit, native.status.code(), "{root:?}");
            assert_eq!(native_report["return_code"], -17, "{root:?}");
            assert_eq!(
                ares_report.unwrap()["return_code"],
                native_report["return_code"]
            );
        }
    }
    assert_eq!(fs::read(&machine_path).unwrap(), machine.as_bytes());
    assert_eq!(fs::read(&input_path).unwrap(), input);
}
