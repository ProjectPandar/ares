#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::{
    artifacts,
    load_settings_cli::{banner_normalized, preset_files},
    native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_load_defaultfila_fills_explicit_gaps_with_first_filename() {
    let Some(fixture) = Fixture::new("sparse") else {
        return;
    };
    let dense = fixture.native("dense", "a.json;a.json;b.json", false, 0);
    for (label, list) in [
        ("interior", "a.json;;b.json"),
        ("leading", ";a.json;b.json"),
    ] {
        let native = fixture.native(label, list, true, 0);
        native.compare(&dense, &fixture.root, label);
        fixture
            .ares(label, list, true, 0)
            .compare(&native, &fixture.root, label);
    }
}

#[test]
fn cli_load_defaultfila_does_not_extend_short_list_to_nozzle_count() {
    let Some(fixture) = Fixture::new("short") else {
        return;
    };
    let path = fixture.root.join("machine.json");
    let mut machine: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for key in [
        "nozzle_diameter",
        "extruder_colour",
        "extruder_offset",
        "default_filament_profile",
    ] {
        let values = machine[key].as_array_mut().unwrap();
        values.push(values[0].clone());
    }
    machine["single_extruder_multi_material"] = "0".into();
    machine["printer_extruder_id"] = serde_json::json!(["1", "2"]);
    machine["printer_extruder_variant"] =
        serde_json::json!(["Direct Drive Standard", "Direct Drive Standard"]);
    fs::write(path, serde_json::to_vec(&machine).unwrap()).unwrap();
    let native_off = fixture.native("off", "a.json", false, 0);
    let native_on = fixture.native("on", "a.json", true, 0);
    native_on.compare(&native_off, &fixture.root, "dense");
    fixture
        .ares("on", "a.json", true, 0)
        .compare(&native_on, &fixture.root, "dense");
}

#[test]
fn cli_load_defaultfila_rejects_list_without_a_filename() {
    let Some(fixture) = Fixture::new("all-empty") else {
        return;
    };
    let native = fixture.native("empty", ";;", true, 251);
    fixture
        .ares("empty", ";;", true, 251)
        .compare(&native, &fixture.root, "all-empty");
}

struct Fixture {
    root: std::path::PathBuf,
    runner: OrcaRunner,
}

impl Fixture {
    fn new(name: &str) -> Option<Self> {
        let runner = OrcaRunner::from_env().unwrap()?;
        let root = tempfile::Builder::new()
            .prefix(&format!("cli-load-defaultfila-{name}-"))
            .tempdir_in(artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
        let mut process: serde_json::Value =
            serde_json::from_slice(&fs::read(&presets[1]).unwrap()).unwrap();
        // Keep only walls to isolate the filament list from surface and infill geometry.
        process["sparse_infill_density"] = "0%".into();
        process["top_shell_layers"] = "0".into();
        process["bottom_shell_layers"] = "0".into();
        fs::write(&presets[1], serde_json::to_vec(&process).unwrap()).unwrap();
        let mut filament: serde_json::Value =
            serde_json::from_slice(&fs::read(&presets[2]).unwrap()).unwrap();
        filament["name"] = "Synthetic filament A".into();
        filament["filament_id"] = "CLI_A".into();
        fs::write(root.join("a.json"), serde_json::to_vec(&filament).unwrap()).unwrap();
        filament["name"] = "Synthetic filament B".into();
        filament["filament_id"] = "CLI_B".into();
        filament["nozzle_temperature"] = serde_json::json!(["220"]);
        fs::write(root.join("b.json"), serde_json::to_vec(&filament).unwrap()).unwrap();
        fs::copy(
            crate::runner::repo_root().join("tests/parity/cube10.stl"),
            root.join("cube10.stl"),
        )
        .unwrap();
        Some(Self { root, runner })
    }

    fn native(&self, label: &str, list: &str, use_default: bool, exit: i32) -> SliceOutput {
        self.run(
            self.runner.bin(),
            ("native", label),
            list,
            (use_default, exit),
        )
    }

    fn ares(&self, label: &str, list: &str, use_default: bool, exit: i32) -> SliceOutput {
        self.run(
            Path::new(env!("CARGO_BIN_EXE_ares")),
            ("ares", label),
            list,
            (use_default, exit),
        )
    }

    fn run(
        &self,
        bin: &Path,
        (engine, label): (&str, &str),
        list: &str,
        (use_default, exit): (bool, i32),
    ) -> SliceOutput {
        let cwd = self.root.join(format!("{engine}-{label}"));
        let out = cwd.join("out");
        fs::create_dir_all(&out).unwrap();
        let absolute_list = list
            .split(';')
            .map(|name| {
                if name.is_empty() {
                    String::new()
                } else {
                    self.root.join(name).to_string_lossy().into_owned()
                }
            })
            .collect::<Vec<_>>()
            .join(";");
        let mut command = Command::new(bin);
        command.current_dir(&cwd).args([
            "--slice",
            "1",
            "--outputdir",
            "out",
            "--load-settings",
            &format!(
                "{};{}",
                self.root.join("machine.json").display(),
                self.root.join("process.json").display()
            ),
            "--load-filaments",
            &absolute_list,
        ]);
        if use_default {
            command.arg("--load-defaultfila");
        }
        let result = command.arg(self.root.join("cube10.stl")).output().unwrap();
        fs::write(cwd.join("exit"), format!("{:?}\n", result.status.code())).unwrap();
        fs::write(cwd.join("stderr"), &result.stderr).unwrap();
        fs::write(cwd.join("stdout"), &result.stdout).unwrap();
        assert_eq!(result.status.code(), Some(exit), "{cwd:?}");
        let mut files = fs::read_dir(&out)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        files.sort_unstable();
        let expected: &[_] = if exit == 0 {
            &["plate_1.gcode", "result.json"]
        } else {
            &["result.json"]
        };
        assert_eq!(files, expected, "{cwd:?}");
        SliceOutput {
            stderr: result.stderr,
            report: result_comparison_bytes(&fs::read(out.join("result.json")).unwrap()),
            gcode: (exit == 0)
                .then(|| banner_normalized(&fs::read(out.join("plate_1.gcode")).unwrap())),
        }
    }
}

struct SliceOutput {
    stderr: Vec<u8>,
    report: Vec<u8>,
    gcode: Option<Vec<u8>>,
}

impl SliceOutput {
    fn compare(&self, expected: &Self, root: &Path, label: &str) {
        assert_eq!(self.stderr, expected.stderr, "stderr at {root:?} {label}");
        assert!(
            self.report == expected.report,
            "result.json at {root:?} {label}"
        );
        assert!(self.gcode == expected.gcode, "G-code at {root:?} {label}");
    }
}
