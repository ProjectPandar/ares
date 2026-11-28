use std::{fs, io::Cursor, path::Path, process::Command};

use serde_json::{Value, json};

use super::archive_members;
use crate::OrcaRunner;

const CONTROL: &[u8] =
    include_bytes!("../../../../../tests/parity/mesh-edge-pairing/cube-control.3mf");
const SETTINGS: &str = "Metadata/project_settings.config";
const TOP_LEGACY: &str = "top_one_wall_type";
const TOP: &str = "only_one_wall_top";

#[test]
fn empty_top_wall_scalar_and_array_enable_top_wall_mode() {
    check(
        "empty-top-wall-",
        TOP_LEGACY,
        &[("empty-string", json!("")), ("empty-array", json!([]))],
        TOP,
        (json!("1"), None),
    );
}

#[test]
fn top_wall_array_reports_the_appended_first_pass_token() {
    check(
        "top-wall-array-",
        TOP_LEGACY,
        &[
            ("array-none", json!(["none"])),
            ("scalar-none", json!("none")),
        ],
        TOP,
        (
            json!("0"),
            Some(&[
                json!([{
                    "class": "config_value_substituted",
                    "file": "input.3mf",
                    "key": TOP,
                    "old_value": "1none",
                    "new_value": "0"
                }]),
                json!([]),
            ]),
        ),
    );
}

#[test]
fn feature_array_keeps_explicit_filament_one_after_rename() {
    check(
        "feature-scalar-one-",
        "infill_extruder",
        &[("scalar-one", json!("1"))],
        "sparse_infill_filament_id",
        (json!("0"), None),
    );
    check(
        "feature-array-one-",
        "infill_extruder",
        &[("array-one", json!(["1"]))],
        "sparse_infill_filament_id",
        (json!("1"), None),
    );
}

#[test]
fn arc_fitting_array_binds_a_rewritten_scalar_enum() {
    check(
        "arc-fitting-array-",
        "enable_arc_fitting",
        &[("array-zero", json!(["0"]))],
        "enable_arc_fitting",
        (json!("disabled"), None),
    );
}

#[test]
fn filament_type_array_rewrites_the_legacy_vector_member() {
    check(
        "filament-type-array-",
        "filament_type",
        &[("array-asa-aero", json!(["ASA-Aero"]))],
        "filament_type",
        (json!(["ASA-AERO"]), None),
    );
}

#[test]
fn extruder_type_array_rewrites_before_enum_substitution() {
    check(
        "extruder-type-array-",
        "extruder_type",
        &[("array-direct-drive", json!(["DirectDrive"]))],
        "extruder_type",
        (json!(["Direct Drive"]), Some(&[json!([])])),
    );
}

#[test]
fn bookkeeping_array_rewrites_names_without_losing_slots() {
    check(
        "bookkeeping-array-",
        "different_settings_to_system",
        &[(
            "array-legacy",
            json!(["bridge_fan_speed;layer_height", "", ""]),
        )],
        "different_settings_to_system",
        (json!(["overhang_fan_speed;layer_height", "", ""]), None),
    );
}

fn check(
    prefix: &str,
    source: &str,
    variants: &[(&str, Value)],
    destination: &str,
    expected: (Value, Option<&[Value]>),
) {
    let (expected, expected_warnings) = expected;
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(runner.work_dir())
        .unwrap()
        .keep();
    let control = project(source, destination, destination, &expected);
    let native_control = observe(
        runner.bin(),
        &root.join("native-control"),
        &control,
        destination,
        expected_warnings.is_some(),
    );
    assert_eq!(native_control.saved, expected, "Native control at {root:?}");
    if let Some(warnings) = &native_control.warnings {
        assert_eq!(*warnings, json!([]), "Native control at {root:?}");
    }

    let inputs = variants
        .iter()
        .map(|(label, value)| (*label, project(source, destination, source, value)))
        .collect::<Vec<_>>();
    let native = inputs
        .iter()
        .enumerate()
        .map(|(index, (label, input))| {
            let native = observe(
                runner.bin(),
                &root.join(format!("native-{label}")),
                input,
                destination,
                expected_warnings.is_some(),
            );
            assert_eq!(
                native.saved, native_control.saved,
                "Native {label} at {root:?}"
            );
            assert_eq!(
                native.warnings.as_ref(),
                expected_warnings.map(|warnings| &warnings[index]),
                "Native {label} warnings at {root:?}"
            );
            native
        })
        .collect::<Vec<_>>();
    let actual = inputs
        .iter()
        .map(|(label, input)| {
            observe(
                Path::new(env!("CARGO_BIN_EXE_ares")),
                &root.join(format!("ares-{label}")),
                input,
                destination,
                expected_warnings.is_some(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual, native,
        "saved {destination} and slice warnings at {root:?}"
    );
}

#[derive(Debug, PartialEq)]
struct Observation {
    saved: Value,
    warnings: Option<Value>,
}

fn observe(bin: &Path, dir: &Path, input: &[u8], key: &str, slice: bool) -> Observation {
    fs::create_dir(dir).unwrap();
    fs::write(dir.join("input.3mf"), input).unwrap();
    run(
        bin,
        dir,
        "export",
        &[
            "--arrange",
            "0",
            "--orient",
            "0",
            "--export-3mf",
            "saved.3mf",
            "input.3mf",
        ],
    );
    let settings: Value = archive_members(&dir.join("saved.3mf"))
        .into_iter()
        .find_map(|(name, bytes)| {
            (name == SETTINGS).then(|| serde_json::from_slice(&bytes).unwrap())
        })
        .unwrap();
    let warnings = slice.then(|| {
        fs::create_dir(dir.join("out")).unwrap();
        run(
            bin,
            dir,
            "slice",
            &["--slice", "1", "--outputdir", "out", "input.3mf"],
        );
        assert!(dir.join("out/plate_1.gcode").is_file());
        let mut result: Value =
            serde_json::from_slice(&fs::read(dir.join("out/result.json")).unwrap()).unwrap();
        assert_eq!(result["return_code"], 0);
        let mut warnings = result["warnings"].take();
        for warning in warnings.as_array_mut().unwrap() {
            let file = Path::new(warning["file"].as_str().unwrap());
            assert!(file == Path::new("input.3mf") || file == dir.join("input.3mf"));
            warning["file"] = json!("input.3mf");
        }
        warnings
    });
    Observation {
        saved: settings[key].clone(),
        warnings,
    }
}

fn run(bin: &Path, dir: &Path, stage: &str, args: &[&str]) {
    let output = Command::new(bin)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    fs::write(dir.join(format!("{stage}.stdout")), &output.stdout).unwrap();
    fs::write(dir.join(format!("{stage}.stderr")), &output.stderr).unwrap();
    assert!(output.status.success(), "{stage} failed at {dir:?}");
}

fn project(source: &str, destination: &str, key: &str, value: &Value) -> Vec<u8> {
    let mut original = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..original.len() {
        let mut member = original.by_index(index).unwrap();
        archive
            .start_file(member.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        if member.name() == SETTINGS {
            let mut fields: serde_json::Map<String, Value> =
                serde_json::from_reader(member).unwrap();
            fields.remove(source);
            fields.remove(destination);
            fields.insert(key.to_owned(), value.clone());
            serde_json::to_writer(&mut archive, &fields).unwrap();
        } else {
            std::io::copy(&mut member, &mut archive).unwrap();
        }
    }
    archive.finish().unwrap().into_inner()
}
