#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use super::direct_stl_slice::{inventory, normalized_gcode, normalized_report, square_stl};
use crate::{artifacts, load_settings_cli::preset_files, runner::OrcaRunner};

#[test]
fn cli_direct_stl_slices_all_arranged_plates() {
    compare(0, &[1, 2], None);
}

#[test]
fn cli_direct_stl_slices_only_the_second_arranged_plate() {
    compare(2, &[2], None);
}

#[test]
fn cli_direct_stl_reports_an_unmatched_positive_selector() {
    compare(3, &[], None);
}

#[test]
fn cli_direct_stl_reports_the_malformed_input_before_a_later_missing_file() {
    compare(0, &[], Some(("broken.stl", Some(b"not a mesh\n"))));
}

#[test]
fn cli_direct_stl_reports_the_empty_input() {
    compare(0, &[], Some(("empty.stl", Some(b""))));
}

#[test]
fn cli_direct_stl_reports_a_missing_later_input() {
    compare(0, &[], Some(("missing.stl", None)));
}

fn compare(selection: u32, plates: &[u32], invalid: Option<(&str, Option<&[u8]>)>) {
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
        fs::write(&inputs[1], square_stl([0.0, 0.0], 112.0, 0.6)).unwrap();
        (0, selection)
    };
    let run = |bin: &Path, name: &str| {
        let cwd = root.join(name);
        let dir = cwd.join("out");
        let tmp = cwd.join("tmp");
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir(&tmp).unwrap();
        let output = Command::new(bin)
            .env("TMPDIR", tmp)
            .current_dir(&cwd)
            .args(["--slice", &selection.to_string(), "--outputdir", "out"])
            .args(["--load-settings", &presets[..2].join(";")])
            .args(["--load-filaments", &presets[2..].join(";")])
            .args(&inputs)
            .output()
            .unwrap();
        fs::write(cwd.join("exit"), format!("{:?}\n", output.status.code())).unwrap();
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
