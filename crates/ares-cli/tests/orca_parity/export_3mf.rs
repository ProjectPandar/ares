use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{OrcaRunner, presets::VendorProfiles, runner::repo_root};

#[test]
fn export_x1c_stl_matches_native_unsliced_project_members() {
    export_matches_native("BBL", "Bambu Lab X1 Carbon 0.4 nozzle", "3mf-export.stl");
}

#[test]
fn export_stinger_3mf_matches_native_unsliced_project_members() {
    export_matches_native("LH", "LH Stinger 0.4 nozzle", "3mf-export.3mf");
}

#[test]
#[cfg(unix)]
fn export_rejects_slicing_and_does_not_create_an_archive() {
    let work = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(work.path())
        .args([
            "slice",
            "--export-3mf",
            "sliced.3mf",
            "--slice",
            "0",
            "input.stl",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(254));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("Invalid parameters to the slicer."),
        "{error}"
    );
    assert!(
        error.contains("unsupported 3MF export option --slice"),
        "{error}"
    );
    assert!(!work.path().join("sliced.3mf").exists());
}

fn export_matches_native(vendor: &str, printer: &str, filename: &str) {
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
    let model = if filename.ends_with(".stl") {
        fs::read(repo_root().join("tests/parity/cube10.stl")).unwrap()
    } else {
        synthetic_3mf()
    };
    fs::write(runner.work_dir().join(filename), model).unwrap();
    let settings = [
        "--load-settings",
        "machine.json;process.json",
        "--load-filaments",
        "filament.json",
    ];
    let native = Command::new(runner.bin())
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
            "native.3mf",
        ])
        .arg(filename)
        .output()
        .unwrap();
    assert!(
        native.status.success(),
        "Native export failed in {}: {}",
        runner.work_dir().display(),
        String::from_utf8_lossy(&native.stderr)
    );
    let expected = archive_members(&runner.work_dir().join("native.3mf"));

    let mut first = None;
    for output in ["first.3mf", "second.3mf"] {
        let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
            .current_dir(runner.work_dir())
            .arg("slice")
            .args(settings)
            .arg("--export-3mf")
            .arg(output)
            .arg(filename)
            .output()
            .unwrap();
        assert!(
            ares.status.success(),
            "export failed in {}: {}",
            runner.work_dir().display(),
            String::from_utf8_lossy(&ares.stderr)
        );
        let path = runner.work_dir().join(output);
        let actual = archive_members(&path);
        assert_eq!(
            expected.iter().map(|(name, _)| name).collect::<Vec<_>>(),
            actual.iter().map(|(name, _)| name).collect::<Vec<_>>()
        );
        for ((name, expected), (_, actual)) in expected.iter().zip(actual) {
            let mismatch = expected
                .iter()
                .zip(&actual)
                .position(|(left, right)| left != right);
            assert!(
                mismatch.is_none() && expected.len() == actual.len(),
                "{name} differs in {} at byte {mismatch:?} ({} != {} bytes)",
                runner.work_dir().display(),
                expected.len(),
                actual.len()
            );
        }
        let bytes = fs::read(path).unwrap();
        if let Some(first) = first {
            assert!(first == bytes, "repeat export changed ZIP bytes");
        }
        first = Some(bytes);
    }
}

fn archive_members(path: &Path) -> Vec<(String, Vec<u8>)> {
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

fn synthetic_3mf() -> Vec<u8> {
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
        zip.write_all(body.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
