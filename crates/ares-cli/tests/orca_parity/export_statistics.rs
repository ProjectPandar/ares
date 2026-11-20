#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::Value;

use super::direct_stl_slice::block_stl;
use crate::{CaseInputs, artifacts, load_settings_cli::preset_files, runner::OrcaRunner};

#[test]
fn cli_export_statistics_for_one_stl_object() {
    compare(1, false, false);
}

#[test]
fn cli_export_statistics_for_two_stl_objects() {
    compare(2, false, false);
}

#[test]
fn cli_export_statistics_for_saved_project_plate() {
    compare(1, true, false);
}

#[test]
fn cli_export_statistics_write_failure_leaves_gcode() {
    compare(1, false, true);
}

fn compare(count: usize, project: bool, unwritable: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("export-statistics-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let vendor = "Creality";
    let printer = "Creality Ender-3 0.4 nozzle";
    let presets = preset_files(&root, vendor, printer);
    let settings = presets[..2].join(";");
    let filaments = presets[2..].join(";");
    let stls = (0..count)
        .map(|index| {
            let path = root.join(format!("block-{index}.stl"));
            fs::write(&path, block_stl(108.0 + 24.0 * index as f32)).unwrap();
            path
        })
        .collect::<Vec<_>>();
    let project_path = root.join("project.3mf");
    if project {
        let read = |path: &str| serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: "statistics/project",
                    machine: &read(&presets[0]),
                    process: &read(&presets[1]),
                    filaments: &presets[2..]
                        .iter()
                        .map(|path| read(path))
                        .collect::<Vec<_>>(),
                },
                &Default::default(),
                &stls[0],
            )
            .unwrap();
        fs::write(&project_path, exported.project).unwrap();
    }
    let run = |bin: &Path, name: &str, export: bool| {
        let cwd = root.join(name);
        let out = cwd.join("out");
        fs::create_dir_all(&out).unwrap();
        if unwritable {
            fs::create_dir(out.join("plate_1.statistics.json")).unwrap();
        }
        let mut command = Command::new(bin);
        command
            .current_dir(&cwd)
            .args(["--slice", "1", "--outputdir", "out"]);
        if export {
            command.arg("--export-statistics");
        }
        if project {
            command.arg(&project_path);
        } else {
            command.args(["--load-settings", &settings, "--load-filaments", &filaments]);
            if count > 1 {
                command.args(["--arrange", "0"]);
            }
            command.args(&stls);
        }
        let result = command.output().unwrap();
        fs::write(cwd.join("exit"), format!("{:?}\n", result.status.code())).unwrap();
        fs::write(cwd.join("stderr"), &result.stderr).unwrap();
        (out, result.status.code())
    };
    let (native, status) = run(runner.bin(), "native", true);
    assert_eq!(status, Some(if unwritable { 231 } else { 0 }), "{root:?}");
    let expected = (!unwritable).then(|| statistics(&native, "OrcaSlicer 2.5.0"));
    if let Some(expected) = &expected {
        assert_eq!(
            expected["objects"].as_array().unwrap().len(),
            count,
            "{root:?}"
        );
        assert!(
            expected["summary"]["layer_count"].as_u64().unwrap() > 1,
            "{root:?}"
        );
    }
    let cli = Path::new(env!("CARGO_BIN_EXE_ares"));
    let (ares, ares_status) = run(cli, "ares", true);
    assert_eq!(ares_status, status, "{root:?}");
    if let Some(expected) = expected {
        let actual = statistics(&ares, concat!("Ares ", env!("CARGO_PKG_VERSION")));
        assert!(actual == expected, "complete statistics differ at {root:?}");
        let (control, status) = run(cli, "ares-no-statistics", false);
        assert_eq!(status, Some(0), "{root:?}");
        assert!(
            !control.join("plate_1.statistics.json").exists(),
            "{root:?}"
        );
        assert!(
            fs::read(ares.join("plate_1.gcode")).unwrap()
                == fs::read(control.join("plate_1.gcode")).unwrap(),
            "statistics changed G-code at {root:?}"
        );
    } else {
        assert!(ares.join("plate_1.gcode").is_file(), "{root:?}");
        for dir in [&native, &ares] {
            let result: Value =
                serde_json::from_slice(&fs::read(dir.join("result.json")).unwrap()).unwrap();
            assert_eq!(result["return_code"], -25, "{root:?}");
            assert_eq!(
                result["error_string"], "Failed exporting the G-code statistics.",
                "{root:?}"
            );
        }
    }
}

fn statistics(dir: &Path, identity: &str) -> Value {
    let mut value: Value =
        serde_json::from_slice(&fs::read(dir.join("plate_1.statistics.json")).unwrap()).unwrap();
    assert_eq!(value["generator"], identity, "{dir:?}");
    value.as_object_mut().unwrap().remove("generator");
    value
}
