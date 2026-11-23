#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path, process::Command};

use super::direct_stl_slice::{inventory, normalized_gcode, normalized_report, square_stl};
use crate::{artifacts, load_settings_cli::preset_files, runner::OrcaRunner};

#[test]
fn cli_direct_stl_slices_all_arranged_plates() {
    compare(0, &[1, 2], None, None);
}

#[test]
fn cli_direct_stl_slices_only_the_second_arranged_plate() {
    compare(2, &[2], None, None);
}

#[test]
fn cli_direct_stl_prechecks_the_second_arranged_plate_before_exporting_the_first() {
    compare(1, &[1], None, Some(12));
    compare(0, &[], None, Some(12));
}

#[test]
fn cli_direct_stl_keeps_the_first_export_when_the_second_plate_has_a_path_conflict() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("direct-stl-arranged-conflict-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let mut process: serde_json::Value =
        serde_json::from_slice(&fs::read(&presets[1]).unwrap()).unwrap();
    // Positive contour compensation expands the two neighboring paths after arrangement.
    process["xy_contour_compensation"] = "2".into();
    fs::write(&presets[1], serde_json::to_vec(&process).unwrap()).unwrap();
    let inputs = [("large", 180.0), ("medium", 102.0), ("small", 99.0)].map(|(name, edge)| {
        let path = root.join(format!("{name}.stl"));
        fs::write(&path, square_stl([0.0, 0.0], edge, 1.2)).unwrap();
        OsString::from(path)
    });
    for (label, actions, no_check) in [
        ("checked", &["--slice", "0"][..], false),
        ("before", &["--no-check", "--slice", "0"], true),
        ("after", &["--slice", "0", "--no-check"], false),
    ] {
        let root = root.join(label);
        fs::create_dir(&root).unwrap();
        let mut argv = ["--debug", "2", "--outputdir", "out"]
            .into_iter()
            .chain(actions.iter().copied())
            .map(OsString::from)
            .collect::<Vec<_>>();
        argv.extend([
            "--load-settings".into(),
            presets[..2].join(";").into(),
            "--load-filaments".into(),
            presets[2..].join(";").into(),
        ]);
        argv.extend(inputs.iter().cloned());
        fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
        compare_path_conflict(&runner, &root, &argv, no_check);
    }
}

fn compare_path_conflict(runner: &OrcaRunner, root: &Path, argv: &[OsString], no_check: bool) {
    let native = super::cli_no_check::run(runner.bin(), root, "native", argv);
    let ares = super::cli_no_check::run(Path::new(env!("CARGO_BIN_EXE_ares")), root, "ares", argv);
    let native_dir = root.join("native/cwd/out");
    let ares_dir = root.join("ares/cwd/out");
    let expected = if no_check {
        &["plate_1.gcode", "plate_2.gcode", "result.json"][..]
    } else {
        &["plate_1.gcode", "result.json"][..]
    };
    let report = normalized_report(&native_dir);
    assert_eq!(
        native.status.code(),
        Some(if no_check { 0 } else { 155 }),
        "{root:?}"
    );
    assert_eq!(
        report["return_code"],
        if no_check { 0 } else { -101 },
        "{root:?}"
    );
    assert_eq!(
        report["plate_index"],
        if no_check { 0 } else { 2 },
        "{root:?}"
    );
    assert_eq!(inventory(&native_dir), expected, "{root:?}");
    assert!(
        fs::metadata(native_dir.join("plate_1.gcode"))
            .unwrap()
            .len()
            > 0,
        "{root:?}"
    );
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_dir), expected, "{root:?}");
    assert!(
        fs::metadata(ares_dir.join("plate_1.gcode")).unwrap().len() > 0,
        "{root:?}"
    );
    assert_eq!(normalized_report(&ares_dir), report, "{root:?}");
}

#[test]
fn cli_direct_stl_reports_an_unmatched_positive_selector() {
    compare(3, &[], None, None);
}

#[test]
fn cli_direct_stl_reports_the_malformed_input_before_a_later_missing_file() {
    compare(0, &[], Some(("broken.stl", Some(b"not a mesh\n"))), None);
}

#[test]
fn cli_direct_stl_reports_the_empty_input() {
    compare(0, &[], Some(("empty.stl", Some(b""))), None);
}

#[test]
fn cli_direct_stl_reports_a_missing_later_input() {
    compare(0, &[], Some(("missing.stl", None)), None);
}

fn compare(
    selection: u32,
    plates: &[u32],
    invalid: Option<(&str, Option<&[u8]>)>,
    triangle_limit: Option<u32>,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("direct-stl-plate-action-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let mut inputs = vec![root.join("block-0.stl")];
    fs::write(&inputs[0], square_stl([0.0, 0.0], 115.0, 0.6)).unwrap();
    let (code, reported_plate) = if let Some((name, bytes)) = invalid {
        inputs.push(root.join(name));
        if let Some(bytes) = bytes {
            fs::write(&inputs[1], bytes).unwrap();
            if name == "broken.stl" {
                inputs.push(root.join("missing.stl"));
            }
            (250, 0)
        } else {
            (253, 0)
        }
    } else {
        inputs.push(root.join("block-1.stl"));
        let mut mesh = square_stl([0.0, 0.0], 112.0, 0.6);
        if triangle_limit.is_some() {
            // Split one top facet without changing the object's footprint.
            let facet = 84 + 2 * 50;
            let vertices = [
                mesh[facet + 12..facet + 24].to_vec(),
                mesh[facet + 24..facet + 36].to_vec(),
                mesh[facet + 36..facet + 48].to_vec(),
            ];
            let center = [70.0_f32, 35.0, 0.6].map(f32::to_le_bytes).concat();
            let mut faces = Vec::new();
            for i in 0..3 {
                faces.extend([0; 12]);
                faces.extend(&vertices[i]);
                faces.extend(&vertices[(i + 1) % 3]);
                faces.extend(&center);
                faces.extend([0; 2]);
            }
            mesh.splice(facet..facet + 50, faces);
            mesh[80..84].copy_from_slice(&14_u32.to_le_bytes());
        }
        fs::write(&inputs[1], mesh).unwrap();
        if triangle_limit.is_some() && selection == 0 {
            (197, 2)
        } else {
            (0, selection)
        }
    };
    let run = |bin: &Path, name: &str| {
        let cwd = root.join(name);
        let dir = cwd.join("out");
        let tmp = cwd.join("tmp");
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir(&tmp).unwrap();
        let mut command = Command::new(bin);
        if let Some(limit) = triangle_limit {
            command.args(["--mtcpp", &limit.to_string()]);
        }
        command
            .env("TMPDIR", tmp)
            .current_dir(&cwd)
            .args(["--slice", &selection.to_string(), "--outputdir", "out"])
            .args(["--load-settings", &presets[..2].join(";")])
            .args(["--load-filaments", &presets[2..].join(";")])
            .args(&inputs);
        let argv = command.get_args().collect::<Vec<_>>();
        fs::write(cwd.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
        let output = command.output().unwrap();
        fs::write(cwd.join("exit"), format!("{:?}\n", output.status.code())).unwrap();
        fs::write(cwd.join("stdout"), &output.stdout).unwrap();
        fs::write(cwd.join("stderr"), &output.stderr).unwrap();
        (dir, output)
    };
    let expected_files = plates
        .iter()
        .map(|id| format!("plate_{id}.gcode"))
        .chain(["result.json".to_owned()])
        .collect::<Vec<_>>();
    let (native_dir, native) = run(runner.bin(), "native");
    assert_eq!(native.status.code(), Some(code), "{root:?}");
    assert_eq!(inventory(&native_dir), expected_files, "{root:?}");
    let report = normalized_report(&native_dir);
    assert_eq!(report["plate_index"], reported_plate, "{root:?}");
    if code == 197 {
        assert_eq!(report["return_code"], -59, "{root:?}");
        assert!(native.stderr.is_empty(), "{root:?}");
    }
    if plates.is_empty() {
        assert!(report.get("sliced_plates").is_none(), "{root:?}");
    } else {
        let ids = report["sliced_plates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|plate| plate["id"].as_u64().unwrap() as u32)
            .collect::<Vec<_>>();
        assert_eq!(ids, plates, "{root:?}");
    }
    let (ares_dir, ares) = run(Path::new(env!("CARGO_BIN_EXE_ares")), "ares");
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert!(ares.stderr == native.stderr, "stderr differs at {root:?}");
    assert_eq!(inventory(&ares_dir), expected_files, "{root:?}");
    assert_eq!(normalized_report(&ares_dir), report, "{root:?}");
    for file in expected_files
        .iter()
        .filter(|file| file.ends_with(".gcode"))
    {
        assert!(
            normalized_gcode(&ares_dir.join(file)) == normalized_gcode(&native_dir.join(file)),
            "{file} differs at {root:?}"
        );
    }
}
