use std::{ffi::OsString, fs, path::Path};

use serde_json::{Value, json};

use super::super::cli_no_check::run;
use super::{PROJECT, compare, nonprintable_second_plate, outputs};
use crate::{artifacts, load_settings_3mf::load_settings_3mf_fixture, runner::OrcaRunner};

const SINGLE: &[u8] =
    include_bytes!("../../../../../tests/parity/custom-gcode-overview/overview.3mf");

#[test]
fn direct_slice_zero_exports_the_only_plate_without_a_data_root() {
    verify("0", SINGLE, Data::Omitted, 0, &["plate_1.gcode"]);
}

#[test]
fn direct_slice_zero_exports_all_plates_and_creates_the_data_root() {
    verify(
        "0",
        PROJECT,
        Data::New,
        0,
        &["plate_1.gcode", "plate_2.gcode"],
    );
}

#[test]
fn direct_slice_positive_keeps_the_selected_plate_with_a_data_root() {
    verify("2", PROJECT, Data::New, 0, &["plate_2.gcode"]);
}

#[test]
fn direct_slice_zero_rejects_a_later_invalid_plate_before_any_export() {
    verify("0", &nonprintable_second_plate(), Data::New, 206, &[]);
}

#[test]
fn direct_slice_does_not_start_when_the_data_root_cannot_be_created() {
    verify("0", SINGLE, Data::Blocked, 254, &[]);
}

#[test]
fn direct_slice_uses_the_data_root_to_resolve_a_replacement_preset() {
    let input = fs::read(load_settings_3mf_fixture("compatible-input.3mf")).unwrap();
    verify("1", &input, Data::UserPreset, 0, &["plate_1.gcode"]);
}

#[derive(Clone, Copy)]
enum Data {
    Omitted,
    New,
    Blocked,
    UserPreset,
}

const CHILD: &str = "data/user/chosen/machine/child.json";

impl Data {
    fn prepare(self, cwd: &Path) {
        match self {
            Self::Blocked => fs::write(cwd.join("blocked"), b"").unwrap(),
            Self::UserPreset => {
                let root = cwd.join("data");
                let base = root.join("user/chosen/machine/base");
                fs::create_dir_all(&base).unwrap();
                fs::create_dir(root.join("system")).unwrap();
                let mut parent: Value = serde_json::from_slice(
                    &fs::read(load_settings_3mf_fixture("machine.json")).unwrap(),
                )
                .unwrap();
                parent["version"] = "2.5.0".into();
                parent["from"] = "user".into();
                let parent_name = parent["name"].as_str().unwrap();
                fs::write(base.join(format!("{parent_name}.json")), parent.to_string()).unwrap();
                let mut child: Value = serde_json::from_slice(
                    &fs::read(load_settings_3mf_fixture("user-machine.json")).unwrap(),
                )
                .unwrap();
                child["version"] = "2.5.0".into();
                fs::write(cwd.join(CHILD), child.to_string()).unwrap();
                fs::write(
                    root.join("OrcaSlicer.conf"),
                    json!({"app":{"preset_folder":"chosen"}}).to_string(),
                )
                .unwrap();
            }
            Self::Omitted | Self::New => {}
        }
    }
}

fn verify(selection: &str, input: &[u8], data: Data, exit: i32, names: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-direct-slice-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = root.join("project.3mf");
    fs::write(&input_path, input).unwrap();
    let mut argv: Vec<OsString> = ["--slice", selection, "--outputdir", "out"]
        .map(OsString::from)
        .into();
    match data {
        Data::New | Data::UserPreset => argv.extend(["--datadir".into(), "data".into()]),
        Data::Blocked => argv.extend(["--datadir".into(), "blocked/child".into()]),
        Data::Omitted => {}
    }
    if matches!(data, Data::UserPreset) {
        let process = load_settings_3mf_fixture("process.json");
        let filament = load_settings_3mf_fixture("compatible-filament-0.json");
        argv.extend([
            OsString::from("--load-settings"),
            format!("{CHILD};{}", process.display()).into(),
            "--load-filaments".into(),
            filament.into(),
        ]);
    }
    argv.push(input_path.into());
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let mut references = Vec::new();
    for (name, bin) in [
        ("native", runner.bin().as_path()),
        ("native-replay", runner.bin().as_path()),
        ("ares", Path::new(env!("CARGO_BIN_EXE_ares"))),
    ] {
        fs::create_dir(root.join("cwd")).unwrap();
        data.prepare(&root.join("cwd"));
        let result = run(bin, &root, name, &argv);
        let work = root.join(name).join("cwd");
        assert_eq!(result.status.code(), Some(exit), "{work:?}");
        match data {
            Data::New => assert_eq!(fs::read_dir(work.join("data")).unwrap().count(), 0),
            Data::Omitted => assert!(!work.join("data").exists(), "{work:?}"),
            Data::Blocked => {
                assert_eq!(
                    result.stderr,
                    b"Could not create data directory: blocked/child\nsetup params error\n",
                    "{work:?}"
                );
                assert_eq!(fs::read_dir(work.join("out")).unwrap().count(), 0);
                continue;
            }
            Data::UserPreset => {}
        }
        let result = outputs(&work, true, names);
        if exit == 0 {
            let report: Value = serde_json::from_slice(&result.1).unwrap();
            assert_eq!(report["plate_index"], selection.parse::<u32>().unwrap());
            let ids = report["sliced_plates"].as_array().unwrap();
            assert_eq!(ids.len(), names.len());
            for (plate, name) in ids.iter().zip(names) {
                assert_eq!(format!("plate_{}.gcode", plate["id"]), *name);
            }
        }
        if let Some(native) = references.first() {
            compare(native, &result, name == "native-replay", &root);
        }
        references.push(result);
    }
}
