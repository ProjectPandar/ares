#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path, process::Command};

use super::{
    cli_no_check::{inventory, run},
    direct_stl_slice::square_stl,
};

use crate::{
    artifacts,
    load_settings_cli::{banner_normalized, preset_files},
    native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_convert_unit_prefers_meters_for_tiny_stl() {
    compare("meters", 0.02, 156, true);
}

#[test]
fn cli_convert_unit_infers_inches_for_small_stl() {
    compare("inches", 1.0, 0, true);
}

#[test]
fn cli_convert_unit_leaves_millimeter_stl_unchanged() {
    compare("millimeters", 10.0, 0, false);
}

#[test]
fn cli_convert_unit_scales_all_stls_before_arranging_selected_plate() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-convert-unit-selected-plate-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let settings = presets[..2].join(";");
    let inputs = |unit, edges: [f32; 2], height| {
        edges.map(|edge| {
            let path = root.join(format!("{unit}-{edge}.stl"));
            fs::write(&path, square_stl([0.0, 0.0], edge, height)).unwrap();
            path.into_os_string()
        })
    };
    let meters = inputs("m", [0.13, 0.129], 0.0008);
    let millimeters = inputs("mm", [130.0, 129.0], 0.8);
    for (label, files, flag, selected_output) in [
        ("unconverted", &meters, None, false),
        ("millimeters", &millimeters, None, true),
        ("converted", &meters, Some("--convert-unit"), true),
    ] {
        let argv = [
            "--debug",
            "0",
            "--outputdir",
            "out",
            "--slice",
            "2",
            "--arrange",
            "1",
            "--load-settings",
            &settings,
            "--load-filaments",
            &presets[2],
        ]
        .into_iter()
        .chain(flag)
        .map(OsString::from)
        .chain(files.iter().cloned())
        .collect::<Vec<_>>();
        fs::write(
            root.join(format!("{label}-argv.json")),
            serde_json::to_vec(&argv).unwrap(),
        )
        .unwrap();
        for (engine, bin) in [
            ("native", runner.bin().as_path()),
            ("ares", Path::new(env!("CARGO_BIN_EXE_ares"))),
        ] {
            let name = format!("{label}-{engine}");
            let output = run(bin, &root, &name, &argv);
            assert_eq!(output.status.code(), Some(0), "{name}: {root:?}");
            let cwd = root.join(name).join("cwd");
            let expected = if selected_output {
                &["out/", "out/plate_2.gcode", "out/result.json"][..]
            } else {
                &["out/", "out/result.json"][..]
            };
            assert_eq!(inventory(&cwd), expected, "{cwd:?}");
            let report: serde_json::Value =
                serde_json::from_slice(&fs::read(cwd.join("out/result.json")).unwrap()).unwrap();
            assert_eq!(report["return_code"], 0, "{cwd:?}");
            assert_eq!(report["plate_index"], 2, "{cwd:?}");
            if !selected_output {
                continue;
            }
            let gcode = fs::read_to_string(cwd.join("out/plate_2.gcode")).unwrap();
            for line in [
                "; total layer number: 4",
                "; max_z_height: 0.80",
                "; filament used [mm] = 5625.82",
            ] {
                assert!(
                    gcode.lines().any(|actual| actual == line),
                    "missing {line:?}: {cwd:?}"
                );
            }
        }
    }
}

#[derive(PartialEq)]
struct SliceOutput {
    exit: Option<i32>,
    stderr: Vec<u8>,
    report: Vec<u8>,
    gcode: Option<Vec<u8>>,
}

fn compare(name: &str, side: f32, off_exit: i32, changes_gcode: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("cli-convert-unit-{name}-"))
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join(format!("{name}.stl"));
    fs::write(&input, cube_stl(side)).unwrap();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    // Isolate wall geometry from infill and surface toolpath differences.
    let mut process: serde_json::Value =
        serde_json::from_slice(&fs::read(&presets[1]).unwrap()).unwrap();
    process["sparse_infill_density"] = "0%".into();
    process["top_shell_layers"] = "0".into();
    process["bottom_shell_layers"] = "0".into();
    fs::write(&presets[1], serde_json::to_vec(&process).unwrap()).unwrap();
    let run = |bin: &Path, label: &str, convert: bool, exit: i32| {
        let cwd = root.join(label);
        let out = cwd.join("out");
        fs::create_dir_all(&out).unwrap();
        let mut command = Command::new(bin);
        command
            .current_dir(&cwd)
            .args(["--slice", "1", "--outputdir", "out"]);
        command.args([
            "--load-settings",
            &presets[..2].join(";"),
            "--load-filaments",
            &presets[2..].join(";"),
        ]);
        if convert {
            command.arg("--convert-unit");
        }
        let result = command.arg(&input).output().unwrap();
        fs::write(cwd.join("exit"), format!("{:?}\n", result.status.code())).unwrap();
        fs::write(cwd.join("stderr"), &result.stderr).unwrap();
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
            exit: result.status.code(),
            stderr: result.stderr,
            report: result_comparison_bytes(&fs::read(out.join("result.json")).unwrap()),
            gcode: (exit == 0)
                .then(|| banner_normalized(&fs::read(out.join("plate_1.gcode")).unwrap())),
        }
    };
    let native_off = run(runner.bin(), "native-off", false, off_exit);
    let native_on = run(runner.bin(), "native-on", true, 0);
    assert_eq!(
        native_off.gcode != native_on.gcode,
        changes_gcode,
        "{root:?}"
    );
    for (label, convert, native) in [
        ("ares-on", true, native_on),
        ("ares-off", false, native_off),
    ] {
        let ares = run(
            Path::new(env!("CARGO_BIN_EXE_ares")),
            label,
            convert,
            native.exit.unwrap(),
        );
        assert_eq!(ares.stderr, native.stderr, "{root:?} {label}");
        assert_eq!(ares.report, native.report, "{root:?} {label}");
        assert!(
            ares.gcode == native.gcode,
            "G-code differs at {root:?} {label}"
        );
    }
}

fn cube_stl(d: f32) -> Vec<u8> {
    let vertices = [
        [0.0, 0.0, 0.0],
        [d, 0.0, 0.0],
        [d, d, 0.0],
        [0.0, d, 0.0],
        [0.0, 0.0, d],
        [d, 0.0, d],
        [d, d, d],
        [0.0, d, d],
    ];
    let faces = [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ];
    let mut bytes = vec![0; 80];
    bytes.extend((faces.len() as u32).to_le_bytes());
    for face in faces {
        bytes.extend([0; 12]);
        for index in face {
            for coordinate in vertices[index] {
                bytes.extend(coordinate.to_le_bytes());
            }
        }
        bytes.extend([0; 2]);
    }
    bytes
}
