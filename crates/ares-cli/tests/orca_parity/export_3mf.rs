use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{OrcaRunner, presets::VendorProfiles, runner::repo_root};

use super::export_3mf_model_settings::object_record_permutation;

#[path = "export_3mf/comparison_tests.rs"]
mod comparison_tests;
#[path = "export_3mf/legacy_array_dispatch.rs"]
mod legacy_array_dispatch;
#[path = "export_3mf/legacy_feature_filament.rs"]
mod legacy_feature_filament;
#[path = "export_3mf/legacy_recursive_bookkeeping.rs"]
mod legacy_recursive_bookkeeping;

#[test]
fn export_x1c_stl_matches_native_unsliced_project_members() {
    export_matches_native(
        "BBL",
        "Bambu Lab X1 Carbon 0.4 nozzle",
        "3mf-export.stl",
        || fs::read(repo_root().join("tests/parity/cube10.stl")).unwrap(),
    );
}

#[test]
fn export_stinger_3mf_matches_native_unsliced_project_members() {
    export_matches_native(
        "LH",
        "LH Stinger 0.4 nozzle",
        "3mf-export.3mf",
        synthetic_3mf,
    );
}

#[test]
fn export_preserves_fractional_placement_of_a_rotated_mesh_like_native() {
    export_matches_native("LH", "LH Stinger 0.4 nozzle", "rotated.3mf", || {
        tetrahedron_3mf(true)
    });
}

#[test]
#[cfg(unix)]
fn export_with_missing_stl_fails_like_native_without_an_archive() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let dir = runner.work_dir();
    let missing = dir.join("missing.stl");
    let argv = [
        "--slice",
        "0",
        "--export-3mf",
        "sliced.3mf",
        missing.to_str().unwrap(),
    ];
    let invoke = |bin: &Path| {
        let output = Command::new(bin)
            .current_dir(dir)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .args(argv)
            .output()
            .unwrap();
        assert!(!dir.join("sliced.3mf").exists());
        assert!(!dir.join("plate_1.gcode").exists());
        output
    };
    let expected = invoke(runner.bin());
    assert_eq!(expected.status.code(), Some(253));
    let actual = invoke(Path::new(env!("CARGO_BIN_EXE_ares")));
    assert_eq!(
        actual.status.code(),
        expected.status.code(),
        "{}",
        String::from_utf8_lossy(&actual.stderr),
    );
    assert_eq!(actual.stderr, expected.stderr, "stderr bytes differ");
}

fn export_matches_native(
    vendor: &str,
    printer: &str,
    filename: &str,
    model: impl FnOnce() -> Vec<u8>,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), vendor).unwrap();
    let process_name = profiles.compatible_process(printer).unwrap();
    let filament_name = profiles.compatible_filament(printer).unwrap();
    for (kind, mut fields) in [
        ("machine", profiles.machine(printer).unwrap()),
        ("process", profiles.process(&process_name).unwrap()),
        ("filament", profiles.filament(&filament_name).unwrap()),
    ] {
        fields.extend([
            ("type".into(), json!(kind)),
            ("from".into(), json!("system")),
            ("instantiation".into(), json!("true")),
        ]);
        fs::write(
            runner.work_dir().join(format!("{kind}.json")),
            serde_json::to_vec(&Value::Object(fields)).unwrap(),
        )
        .unwrap();
    }
    fs::write(runner.work_dir().join(filename), model()).unwrap();
    let settings = [
        "--load-settings",
        "machine.json;process.json",
        "--load-filaments",
        "filament.json",
    ];
    let path = runner.work_dir().join("export.3mf");
    let invoke = |bin: &Path| {
        Command::new(bin)
            .current_dir(runner.work_dir())
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .args(settings)
            .args([
                "--arrange",
                "0",
                "--orient",
                "0",
                "--export-3mf",
                "export.3mf",
            ])
            .arg(filename)
            .output()
            .unwrap()
    };
    let native = invoke(runner.bin());
    assert!(
        native.status.success(),
        "Native export failed in {}: {}",
        runner.work_dir().display(),
        String::from_utf8_lossy(&native.stderr)
    );
    let expected = archive_members(&path);

    let mut first = None;
    for _ in 0..2 {
        fs::remove_file(&path).unwrap();
        let ares = invoke(Path::new(env!("CARGO_BIN_EXE_ares")));
        assert!(
            ares.status.success(),
            "export failed in {}: {}",
            runner.work_dir().display(),
            String::from_utf8_lossy(&ares.stderr)
        );
        compare_archive_members(&expected, &archive_members(&path), &path);
        let bytes = fs::read(&path).unwrap();
        if let Some(first) = first {
            assert!(first == bytes, "repeat export changed ZIP bytes");
        }
        first = Some(bytes);
    }
}

fn compare_archive_members(
    expected: &[(String, Vec<u8>)],
    actual: &[(String, Vec<u8>)],
    path: &Path,
) {
    assert_eq!(
        expected.iter().map(|(name, _)| name).collect::<Vec<_>>(),
        actual.iter().map(|(name, _)| name).collect::<Vec<_>>()
    );
    for ((name, expected), (_, actual)) in expected.iter().zip(actual) {
        if expected == actual || object_record_permutation(name, expected, actual) {
            continue;
        }
        let mismatch = expected
            .iter()
            .zip(actual)
            .position(|(left, right)| left != right);
        panic!(
            "{name} differs in {} at byte {mismatch:?} ({} != {} bytes)",
            path.display(),
            expected.len(),
            actual.len()
        );
    }
}

pub(super) fn archive_members(path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut zip = ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    (0..zip.len())
        .map(|index| {
            let mut member = zip.by_index(index).unwrap();
            let mut bytes = Vec::new();
            member.read_to_end(&mut bytes).unwrap();
            (member.name().to_owned(), bytes)
        })
        .collect()
}

pub(super) fn synthetic_3mf() -> Vec<u8> {
    tetrahedron_3mf(false)
}

fn tetrahedron_3mf(rotated: bool) -> Vec<u8> {
    let members = [
        (
            "[Content_Types].xml",
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
        ),
        (
            "3D/3dmodel.model",
            r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" name="Tetrahedron" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="0" y="10" z="0"/><vertex x="0" y="0" z="10"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="1" v3="3"/><triangle v1="1" v2="2" v3="3"/><triangle v1="2" v2="0" v3="3"/></triangles></mesh></object></resources><build><item objectid="1" transform="1 0 0 0 1 0 0 0 1 100 100 0"/></build></model>"#,
        ),
    ];
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in members {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        if rotated && name == "3D/3dmodel.model" {
            // Rotate the tetrahedron mesh by a 3-4-5 basis so vertex coordinates remain exact.
            let body = body
                .replace("x=\"10\" y=\"0\"", "x=\"6\" y=\"8\"")
                .replace("x=\"0\" y=\"10\"", "x=\"-8\" y=\"6\"")
                .replace(" 100 100 0", " 100.123456 100.234567 0");
            zip.write_all(body.as_bytes()).unwrap();
        } else {
            zip.write_all(body.as_bytes()).unwrap();
        }
    }
    zip.finish().unwrap().into_inner()
}
