//! PrintObject.cpp:748-804 / ContourZ.cpp:33-159 / GCode.cpp:7012-7030.
//! Public seam failures: inactive Native, flattened XYZ, wrong E scaling,
//! capability rejection, API/CLI divergence and changed disabled control.
use super::fixture::{self, Scope};
use crate::{self as parity, runner::OrcaRunner};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[test]
fn zaa_sloped_roof_preserves_complete_xyz_output() {
    for enabled in [false, true] {
        let Some(runner) = OrcaRunner::from_env().unwrap() else {
            return;
        };
        let (mut exported, fixture_proof) =
            fixture::prepare(&runner, Scope::Object, "rectilinear", false);
        let (project, input_proof) = wedge(&exported.project, enabled);
        exported.project = project;
        exported.label = format!("zaa-contour-z/sloped-roof/enabled-{enabled}");
        fs::write(&exported.project_path, &exported.project).unwrap();
        parity::runner::application::verify_value(
            &exported.project,
            "zaa_enabled",
            &json!(if enabled { "1" } else { "0" }),
        )
        .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let directory = outcome.artifacts.as_ref().unwrap();
        let xyz = observe(&sliced.reference);
        let cli = crate::runner::application::cli_command()
            .arg("slice")
            .arg("-o")
            .arg(directory.join("cli.gcode"))
            .arg(directory.join("input.3mf"))
            .output()
            .unwrap();
        fs::write(directory.join("cli.stdout"), &cli.stdout).unwrap();
        fs::write(directory.join("cli.stderr"), &cli.stderr).unwrap();
        fs::write(
            directory.join("zaa-witness.json"),
            serde_json::to_vec_pretty(&json!({
                "enabled":enabled,"fixture":fixture_proof,"input":input_proof,
                "input_sha256":fixture::hash(&sliced.project),
                "native_sha256":fixture::hash(&sliced.reference),
                "native_xyz":xyz,"status":outcome.status,"detail":outcome.detail,
                "cli_exit":cli.status.code()
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(
            std::str::from_utf8(&sliced.reference)
                .unwrap()
                .contains("; CONFIG_BLOCK_END")
        );
        assert_eq!(xyz.is_empty(), !enabled, "Native activity: {directory:?}");
        if enabled {
            for role in ["Inner wall", "Outer wall", "Top surface"] {
                assert!(
                    xyz.iter().any(|entry| entry["role"] == role),
                    "Native contoured {role}: {directory:?}"
                );
            }
        }
        assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
        assert!(cli.status.success(), "CLI ZAA: {directory:?}");
        let api = fs::read(directory.join("ares.gcode")).unwrap();
        let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
        assert_eq!(api, cli_bytes, "API/CLI raw output: {directory:?}");
        parity::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes)
            .unwrap();
    }
}

pub(super) fn wedge(project: &[u8], enabled: bool) -> (Vec<u8>, serde_json::Value) {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = serde_json::Map::new();
    let mut meshes = 0;
    let mut vertex_count = 0;
    let regex = regex::Regex::new(r#"<vertex x="([^"]+)" y="([^"]+)" z="([^"]+)"/>"#).unwrap();
    let vertex = |capture: &regex::Captures<'_>| {
        let x = capture[1].parse::<f64>().unwrap();
        let y = capture[2].parse::<f64>().unwrap();
        let old_z = capture[3].parse::<f64>().unwrap();
        let z = if old_z < 0.0 { -5.0 } else { x / 5.0 };
        format!("<vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>")
    };
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in [
                ("zaa_enabled", if enabled { "1" } else { "0" }),
                ("wall_generator", "classic"),
                ("brim_type", "no_brim"),
                ("enable_support", "0"),
                ("enable_arc_fitting", "0"),
            ] {
                settings[key] = json!(value);
            }
            bytes = serde_json::to_vec(&settings).unwrap();
        } else if file.name().ends_with(".model") && bytes.windows(10).any(|w| w == b"<vertices>") {
            let text = String::from_utf8(bytes).unwrap();
            vertex_count = regex.find_iter(&text).count();
            bytes = regex.replace_all(&text, vertex).as_bytes().to_vec();
            meshes += 1;
        } else {
            unchanged.insert(file.name().into(), json!(fixture::hash(&bytes)));
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    assert_eq!(meshes, 1);
    assert_eq!(vertex_count, 8);
    let project = output.finish().unwrap().into_inner();
    let mut final_input = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    for (name, expected) in &unchanged {
        let mut bytes = Vec::new();
        final_input
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(json!(fixture::hash(&bytes)), *expected);
    }
    (
        project,
        json!({"unchanged_members":unchanged,"roof":"z=x/5","bottom":-5}),
    )
}

fn observe(bytes: &[u8]) -> Vec<serde_json::Value> {
    let mut role = "";
    let mut xyz = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if line.starts_with("G1 ")
            && (line.contains(" X") || line.contains(" Y"))
            && line.contains(" Z")
            && line
                .split_ascii_whitespace()
                .find_map(|token| token.strip_prefix('E'))
                .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
        {
            xyz.push(json!({"role":role,"line":line}));
        }
    }
    xyz
}
