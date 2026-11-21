#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use crate::{OrcaRunner, VendorProfiles, export_selection_case, runner::repo_root};

#[test]
fn cli_downward_machine_report_matches_native_for_fit_truncation_and_exclude_rules() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-downward-machines-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = write_fractional_project(&runner, &root);
    let cases = [
        ("fit", target(11.6, 20.6, 20.6, None), Some("fit")),
        ("truncation", target(10.6, 20.6, 20.6, None), None),
        (
            "one-axis-exclude",
            target(20.6, 20.6, 20.6, Some((11.6, 9.6))),
            Some("one-axis-exclude"),
        ),
        (
            "both-axis-exclude",
            target(20.6, 20.6, 20.6, Some((11.6, 11.6))),
            None,
        ),
    ];
    for (label, machine, expected) in cases {
        exercise(&root, &input, label, machine, expected);
    }
}

#[test]
fn cli_downward_machine_missing_target_matches_native_error_channels() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-downward-missing-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = write_fractional_project(&runner, &root);
    exercise_error(&root, &input, ("missing", None, 253, -3));
}

#[test]
fn cli_downward_machine_malformed_target_matches_native_error_channels() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-downward-malformed-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = write_fractional_project(&runner, &root);
    exercise_error(&root, &input, ("malformed", Some(b"{"), 251, -5));
}

fn write_fractional_project(runner: &OrcaRunner, root: &Path) -> std::path::PathBuf {
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let selection =
        crate::select_printer(&profiles, "BBL", "Bambu Lab X1 Carbon 0.4 nozzle").unwrap();
    let source = fs::read(repo_root().join("tests/parity/cube10.stl")).unwrap();
    let source = fractional_x_stl(&source);
    let stl = root.join("fractional.stl");
    fs::write(&stl, source).unwrap();
    let exported = export_selection_case(runner, &profiles, &selection, &stl).unwrap();
    let input = root.join("input.3mf");
    fs::write(&input, exported.project).unwrap();
    input
}

fn fractional_x_stl(source: &[u8]) -> Vec<u8> {
    let mut bytes = source.to_vec();
    for facet in bytes[84..].as_chunks_mut::<50>().0 {
        for vertex in facet[12..48].as_chunks_mut::<12>().0 {
            let value = f32::from_le_bytes(vertex[..4].try_into().unwrap());
            vertex[..4].copy_from_slice(&(value * 1.03).to_le_bytes());
        }
    }
    bytes
}

fn target(width: f64, depth: f64, height: f64, exclude: Option<(f64, f64)>) -> Value {
    let exclude = exclude.map_or_else(Vec::new, |(width, depth)| {
        vec![
            json!("0x0"),
            json!(format!("{width}x0")),
            json!(format!("{width}x{depth}")),
            json!(format!("0x{depth}")),
        ]
    });
    json!({
        "name": "Synthetic downward target",
        "type": "machine",
        "from": "system",
        "printable_area": ["0x0", format!("{width}x0"), format!("{width}x{depth}"), format!("0x{depth}")],
        "printable_height": height.to_string(),
        "bed_exclude_area": exclude,
        "wrapping_exclude_area": []
    })
}

fn exercise(root: &Path, input: &Path, label: &str, machine: Value, expected: Option<&str>) {
    let target_path = root.join(format!("{label}.json"));
    fs::write(&target_path, serde_json::to_vec_pretty(&machine).unwrap()).unwrap();
    let native = run_native(root, input, &target_path, label);
    let ares = run_ares(root, input, &target_path, label);
    assert_eq!(native.status.code(), Some(0), "{label}: native");
    assert_eq!(ares.status.code(), native.status.code(), "{label}: exit");
    let native_report = report(root, "native", label);
    let ares_report = report(root, "ares", label);
    assert_eq!(ares_report["return_code"], native_report["return_code"]);
    match expected {
        Some(_) => assert_eq!(
            ares_report["downward_compatible_machine"],
            json!(["Synthetic downward target"])
        ),
        None => assert!(
            !ares_report
                .as_object()
                .unwrap()
                .contains_key("downward_compatible_machine")
        ),
    }
    assert_eq!(
        ares_report.get("downward_compatible_machine"),
        native_report.get("downward_compatible_machine"),
        "{label}: downward report",
    );
    for key in [
        "return_code",
        "layer_height",
        "wall_loops",
        "sparse_infill_density",
        "upward_compatible_machine",
    ] {
        assert_eq!(ares_report[key], native_report[key], "{label}: {key}");
    }
}

fn exercise_error(
    root: &Path,
    input: &Path,
    (label, contents, exit, return_code): (&str, Option<&[u8]>, i32, i32),
) {
    let target_path = root.join(format!("{label}.json"));
    if let Some(contents) = contents {
        fs::write(&target_path, contents).unwrap();
    }
    let native = run_native(root, input, &target_path, label);
    let ares = run_ares(root, input, &target_path, label);
    assert_eq!(native.status.code(), Some(exit), "{label}: native");
    assert_eq!(ares.status.code(), native.status.code(), "{label}: exit");
    assert_eq!(ares.stderr, native.stderr, "{label}: stderr");
    let native_report = report(root, "native", label);
    let ares_report = report(root, "ares", label);
    assert_eq!(native_report["return_code"], return_code);
    for key in [
        "return_code",
        "error_string",
        "plate_index",
        "layer_height",
        "sparse_infill_density",
        "strict_mode",
        "upward_compatible_machine",
        "wall_loops",
        "warnings",
    ] {
        assert_eq!(ares_report[key], native_report[key], "{label}: {key}");
    }
}

fn run_native(root: &Path, input: &Path, target: &Path, label: &str) -> std::process::Output {
    let dir = root.join(format!("native-{label}"));
    fs::create_dir(&dir).unwrap();
    Command::new(std::env::var_os("ARES_ORCA_BIN").unwrap())
        .args([
            "--slice",
            "0",
            "--downward-check",
            "--downward-settings",
            target.to_str().unwrap(),
            "--outputdir",
            dir.to_str().unwrap(),
        ])
        .arg(input)
        .current_dir(root)
        .output()
        .unwrap()
}

fn run_ares(root: &Path, input: &Path, target: &Path, label: &str) -> std::process::Output {
    let dir = root.join(format!("ares-{label}"));
    fs::create_dir(&dir).unwrap();
    Command::new(env!("CARGO_BIN_EXE_ares"))
        .args([
            "slice",
            "--downward-check",
            "--downward-settings",
            target.to_str().unwrap(),
            "-o",
            dir.join("plate_1.gcode").to_str().unwrap(),
        ])
        .arg(input)
        .current_dir(root)
        .output()
        .unwrap()
}

fn report(root: &Path, producer: &str, label: &str) -> Value {
    let path = root.join(format!("{producer}-{label}/result.json"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
