use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{OrcaRunner, presets::VendorProfiles, runner::repo_root};

const FACE: [[usize; 3]; 4] = [[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]];
const POINT: [[f32; 3]; 4] = [[1., 2., 3.], [11., 2., 3.], [1., 12., 3.], [1., 2., 13.]];

#[test]
fn cli_export_stl_matches_native_binary_file_for_stl() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let filename = "Tetra.stl";
    fs::write(runner.work_dir().join(filename), synthetic_stl()).unwrap();
    write_native_presets(runner.work_dir());
    let expected = native(&runner, filename, false, false);
    assert_eq!(names(&expected), ["obj_1_Tetra.stl"]);
    compare(&expected, &ares(&runner, filename, false, false));
}

#[test]
fn cli_export_stls_matches_native_binary_files_in_build_order() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let filename = "Scene.3mf";
    fs::write(
        runner.work_dir().join(filename),
        synthetic_3mf(false, false),
    )
    .unwrap();
    write_native_presets(runner.work_dir());
    let expected = native(&runner, filename, true, false);
    assert_eq!(expected.len(), 2);
    compare(&expected, &ares(&runner, filename, true, false));
}

#[test]
fn cli_repair_removes_redundant_geometry_before_binary_export() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let filename = "Repair.3mf";
    fs::write(runner.work_dir().join(filename), synthetic_3mf(true, false)).unwrap();
    write_native_presets(runner.work_dir());
    let original = native(&runner, filename, true, false);
    let repaired = native(&runner, filename, true, true);
    assert_eq!(original[0].1.len(), 84 + 5 * 50);
    assert_eq!(repaired[0].1.len(), 84 + 4 * 50);
    assert_eq!(original[1], repaired[1]);
    compare(&repaired, &ares(&runner, filename, true, true));
    compare(&original, &ares(&runner, filename, true, false));
}

#[test]
#[cfg(unix)]
fn cli_stl_export_rejects_deferred_actions_geometry_and_write_errors() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Open.3mf"), synthetic_3mf(false, true)).unwrap();
    fs::write(dir.path().join("Valid.stl"), synthetic_stl()).unwrap();
    fs::write(dir.path().join("not-dir"), []).unwrap();
    for (args, code, diagnostic) in [
        (
            &["--repair", "-o", "out.gcode", "Valid.stl"][..],
            254,
            "--repair",
        ),
        (
            &["--export-stl", "--slice", "0", "Valid.stl"][..],
            254,
            "--slice",
        ),
        (
            &["--export-stl", "--export-3mf", "out.3mf", "Valid.stl"][..],
            254,
            "--export-3mf",
        ),
        (
            &["--export-stl", "--arrange", "1", "Valid.stl"][..],
            254,
            "--arrange",
        ),
        (
            &["--export-stl", "--repair", "Open.3mf"][..],
            254,
            "unsupported STL repair",
        ),
        (
            &["--export-stls", "not-dir", "Valid.stl"][..],
            245,
            "Failed exporting STL files.",
        ),
        (&["--export-stl", "Missing.stl"][..], 253, "No such file:"),
    ] {
        let output = ares_command(dir.path()).args(args).output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(code), "{args:?}: {stderr}");
        assert!(stderr.contains(diagnostic), "{args:?}: {stderr}");
        for path in ["stl", "out.gcode", "out.3mf"] {
            assert!(!dir.path().join(path).exists());
        }
    }
}

fn ares_command(dir: &Path) -> Command {
    let mut ares = Command::new(env!("CARGO_BIN_EXE_ares"));
    ares.current_dir(dir).arg("slice");
    ares
}

type Artifacts = Vec<(String, Vec<u8>)>;

fn names(artifacts: &Artifacts) -> Vec<&str> {
    artifacts.iter().map(|(name, _)| name.as_str()).collect()
}

fn compare(expected: &Artifacts, actual: &Artifacts) {
    assert_eq!(names(expected), names(actual));
    for ((name, lhs), (_, rhs)) in expected.iter().zip(actual) {
        assert_eq!(lhs.len(), rhs.len(), "{name}");
        let mismatch = lhs.iter().zip(rhs).position(|(a, b)| a != b);
        assert_eq!(mismatch, None, "{name}: first differing byte {mismatch:?}");
    }
}

fn native(runner: &OrcaRunner, input: &str, plural: bool, repair: bool) -> Artifacts {
    let mut cmd = Command::new(runner.bin());
    cmd.current_dir(runner.work_dir())
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .args([
            "--load-settings",
            "machine.json;process.json",
            "--load-filaments",
            "filament.json",
        ]);
    export(runner.work_dir(), cmd, input, plural, ("native", repair))
}

fn ares(runner: &OrcaRunner, input: &str, plural: bool, repair: bool) -> Artifacts {
    export(
        runner.work_dir(),
        ares_command(runner.work_dir()),
        input,
        plural,
        ("ares", repair),
    )
}

fn export(
    dir: &Path,
    mut cmd: Command,
    input: &str,
    plural: bool,
    (kind, repair): (&str, bool),
) -> Artifacts {
    let root = dir.join(format!("{kind}-{repair}"));
    fs::create_dir(&root).unwrap();
    let files = if plural {
        root.join("custom/deep")
    } else {
        root.join("stl")
    };
    cmd.args(["--arrange", "0", "--orient", "0", "--outputdir"])
        .arg(&root);
    if plural {
        cmd.arg("--export-stls").arg(&files);
    } else {
        cmd.arg("--export-stl");
    }
    if repair {
        cmd.arg("--repair");
    } else {
        cmd.arg("--repair=false");
    }
    let Output {
        status,
        stdout,
        stderr,
    } = cmd.arg(input).output().unwrap();
    fs::write(root.join("stdout.log"), &stdout).unwrap();
    fs::write(root.join("stderr.log"), &stderr).unwrap();
    assert!(
        status.success(),
        "{kind} failed in {}: {} / {}",
        root.display(),
        String::from_utf8_lossy(&stderr),
        String::from_utf8_lossy(&stdout)
    );
    if plural {
        assert!(!root.join("stl").exists());
    }
    let mut entries: Artifacts = fs::read_dir(files)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().into_string().unwrap(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect();
    entries.sort_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
    assert!(!entries.is_empty());
    entries
}

fn write_native_presets(dir: &Path) {
    let printer = "Bambu Lab X1 Carbon 0.4 nozzle";
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    for (kind, mut fields) in [
        ("machine", profiles.machine(printer).unwrap()),
        (
            "process",
            profiles
                .process(&profiles.compatible_process(printer).unwrap())
                .unwrap(),
        ),
        (
            "filament",
            profiles
                .filament(&profiles.compatible_filament(printer).unwrap())
                .unwrap(),
        ),
    ] {
        fields.extend([
            ("type".into(), json!(kind)),
            ("from".into(), json!("system")),
            ("instantiation".into(), json!("true")),
        ]);
        fs::write(
            dir.join(format!("{kind}.json")),
            serde_json::to_vec(&Value::Object(fields)).unwrap(),
        )
        .unwrap();
    }
}

fn synthetic_stl() -> Vec<u8> {
    let mut model = vec![b'X'; 80];
    model.extend(4_u32.to_le_bytes());
    for face in FACE {
        model.extend([0; 12]);
        for point in face.map(|i| POINT[i]) {
            model.extend(point.into_iter().flat_map(f32::to_le_bytes));
        }
        model.extend([0x12, 0x34]);
    }
    model
}

fn synthetic_3mf(dirty: bool, open: bool) -> Vec<u8> {
    let mut xml = String::from(
        r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>"#,
    );
    for (id, name) in [(1, "Beta"), (2, "Alpha.stl")] {
        xml += &format!(r#"<object id="{id}" name="{name}" type="model"><mesh><vertices>"#);
        let mut points = POINT.to_vec();
        let mut faces = FACE.to_vec();
        if id == 1 && dirty {
            points.push(POINT[1]);
            faces[1][1] = 4;
            faces.push(FACE[0]);
        }
        if id == 1 && open {
            faces.pop();
        }
        for [x, y, z] in points {
            xml += &format!(r#"<vertex x="{x}" y="{y}" z="{z}"/>"#);
        }
        xml += "</vertices><triangles>";
        for [v1, v2, v3] in faces {
            xml += &format!(r#"<triangle v1="{v1}" v2="{v2}" v3="{v3}"/>"#);
        }
        xml += "</triangles></mesh></object>";
    }
    xml += r#"</resources><build><item objectid="1" transform="1 0 0 0 1 0 0 0 1 32 44 0"/><item objectid="2" transform="1 0 0 0 1 0 0 0 1 96 55 0"/></build></model>"#;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in [
        (
            "[Content_Types].xml",
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
        ),
        ("3D/3dmodel.model", xml.as_str()),
    ] {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
