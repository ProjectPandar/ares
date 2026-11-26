use super::*;
use crate::{golden, runner::stages};
use std::{
    fmt::Write as _,
    fs,
    io::{Cursor, Read, Write},
};

const BASE_3MF: &[u8] =
    include_bytes!("../../../../../../tests/parity/outer-only-brim/rectangle.3mf");

#[test]
fn clipper_round_skirt_atan2_determinism() {
    let vertices = [
        (-15.0, -10.0, 0.0),
        (15.0, -10.0, 0.0),
        (0.324324, 10.006491, 0.0),
        (-15.0, 12.0, 0.0),
        (-15.0, -10.0, 1.0),
        (15.0, -10.0, 1.0),
        (0.324324, 10.006491, 1.0),
        (-15.0, 12.0, 1.0),
    ];
    let triangles = [
        (3, 1, 0),
        (1, 3, 2),
        (7, 4, 5),
        (5, 6, 7),
        (0, 1, 5),
        (0, 5, 4),
        (1, 2, 6),
        (1, 6, 5),
        (2, 3, 7),
        (2, 7, 6),
        (3, 0, 4),
        (3, 4, 7),
    ];
    let project = build_project(
        &vertices,
        &triangles,
        "110.0 110.0 0.0",
        &[
            ("brim_type", "no_brim"),
            ("brim_width", "0"),
            ("initial_layer_line_width", "0.4"),
            ("line_width", "0.4"),
            ("skirt_distance", "4.8214603125648496"),
            ("skirt_height", "1"),
            ("skirt_loops", "1"),
        ],
    );
    verify_witness(
        "clipper-round-skirt-atan2",
        &project,
        &["G1 X129.539 Y102.096 E.0579", "G1 X129.032 Y102.957 E.0291"],
    );
}

#[test]
fn bridge_normal_bin_atan2_determinism() {
    let vertices = [
        (10.084, -38.0, 0.0),
        (26.0, -38.0, 0.0),
        (26.0, 38.0, 0.0),
        (2.0, 38.0, 0.0),
        (2.0, 12.0, 0.0),
        (20.0, 12.0, 0.0),
        (20.0, -35.769, 0.0),
        (10.084, -35.769, 0.0),
        (10.084, -38.0, 1.0),
        (26.0, -38.0, 1.0),
        (26.0, 38.0, 1.0),
        (2.0, 38.0, 1.0),
        (2.0, 12.0, 1.0),
        (20.0, 12.0, 1.0),
        (20.0, -35.769, 1.0),
        (10.084, -35.769, 1.0),
        (10.084, -38.0, 1.0),
        (26.0, -38.0, 1.0),
        (26.0, 38.0, 1.0),
        (2.0, 38.0, 1.0),
        (2.0, 12.0, 1.0),
        (0.0, 12.0, 1.0),
        (0.0, 10.0, 1.0),
        (2.563503, -8.513382, 1.0),
        (4.563503, -8.513382, 1.0),
        (8.084, -33.769, 1.0),
        (8.084, -35.769, 1.0),
        (10.084, -35.769, 1.0),
        (10.084, -38.0, 2.0),
        (26.0, -38.0, 2.0),
        (26.0, 38.0, 2.0),
        (2.0, 38.0, 2.0),
        (2.0, 12.0, 2.0),
        (0.0, 12.0, 2.0),
        (0.0, 10.0, 2.0),
        (2.563503, -8.513382, 2.0),
        (4.563503, -8.513382, 2.0),
        (8.084, -33.769, 2.0),
        (8.084, -35.769, 2.0),
        (10.084, -35.769, 2.0),
    ];
    let triangles = [
        (7, 1, 0),
        (2, 4, 3),
        (2, 5, 4),
        (1, 5, 2),
        (1, 6, 5),
        (1, 7, 6),
        (15, 8, 9),
        (10, 11, 12),
        (10, 12, 13),
        (9, 10, 13),
        (9, 13, 14),
        (9, 14, 15),
        (0, 1, 9),
        (0, 9, 8),
        (1, 2, 10),
        (1, 10, 9),
        (2, 3, 11),
        (2, 11, 10),
        (3, 4, 12),
        (3, 12, 11),
        (4, 5, 13),
        (4, 13, 12),
        (5, 6, 14),
        (5, 14, 13),
        (6, 7, 15),
        (6, 15, 14),
        (7, 0, 8),
        (7, 8, 15),
        (27, 17, 16),
        (27, 18, 17),
        (27, 19, 18),
        (27, 20, 19),
        (20, 22, 21),
        (20, 23, 22),
        (20, 24, 23),
        (27, 24, 20),
        (27, 25, 24),
        (25, 27, 26),
        (39, 28, 29),
        (39, 29, 30),
        (39, 30, 31),
        (39, 31, 32),
        (32, 33, 34),
        (32, 34, 35),
        (32, 35, 36),
        (39, 32, 36),
        (39, 36, 37),
        (37, 38, 39),
        (16, 17, 29),
        (16, 29, 28),
        (17, 18, 30),
        (17, 30, 29),
        (18, 19, 31),
        (18, 31, 30),
        (19, 20, 32),
        (19, 32, 31),
        (20, 21, 33),
        (20, 33, 32),
        (21, 22, 34),
        (21, 34, 33),
        (22, 23, 35),
        (22, 35, 34),
        (23, 24, 36),
        (23, 36, 35),
        (24, 25, 37),
        (24, 37, 36),
        (25, 26, 38),
        (25, 38, 37),
        (26, 27, 39),
        (26, 39, 38),
        (27, 16, 28),
        (27, 28, 39),
    ];
    let project = build_project(
        &vertices,
        &triangles,
        "110 110 0",
        &[
            ("bottom_shell_layers", "2"),
            ("brim_type", "no_brim"),
            ("brim_width", "0"),
            ("skirt_distance", "6"),
            ("sparse_infill_density", "15%"),
            ("sparse_infill_pattern", "grid"),
            ("top_shell_layers", "2"),
            ("wall_loops", "0"),
        ],
    );
    verify_witness(
        "bridge-normal-bin-atan2",
        &project,
        &[";TYPE:Bridge", "G1 X113.053 Y101.504 Z1.6"],
    );
}

fn build_project(
    vertices: &[(f64, f64, f64)],
    triangles: &[(usize, usize, usize)],
    translation: &str,
    settings_overrides: &[(&str, &str)],
) -> Vec<u8> {
    let mut mesh_xml = String::from("<mesh><vertices>");
    for &(x, y, z) in vertices {
        write!(mesh_xml, r#"<vertex x="{x:.9}" y="{y:.9}" z="{z:.9}"/>"#).unwrap();
    }
    mesh_xml.push_str("</vertices><triangles>");
    for &(v1, v2, v3) in triangles {
        write!(mesh_xml, r#"<triangle v1="{v1}" v2="{v2}" v3="{v3}"/>"#).unwrap();
    }
    mesh_xml.push_str("</triangles></mesh>");

    let mut reader = zip::ZipArchive::new(Cursor::new(BASE_3MF)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut entry = reader.by_index(index).unwrap();
        let name = entry.name().to_owned();
        let compression = entry.compression();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        let content = match name.as_str() {
            "3D/3dmodel.model" => String::from_utf8(bytes)
                .unwrap()
                .replace(
                    r#"transform="1 0 0 0 1 0 0 0 1 110 110 0.5""#,
                    &format!(r#"transform="1 0 0 0 1 0 0 0 1 {translation}""#),
                )
                .into_bytes(),
            "3D/Objects/cube10.stl_1.model" => {
                let text = String::from_utf8(bytes).unwrap();
                let start = text.find("<mesh>").unwrap();
                let end = text.find("</mesh>").unwrap() + "</mesh>".len();
                format!("{}{mesh_xml}{}", &text[..start], &text[end..]).into_bytes()
            }
            "Metadata/project_settings.config" => {
                let mut config: Map<String, Value> = serde_json::from_slice(&bytes).unwrap();
                for &(key, value) in settings_overrides {
                    config.insert(key.to_owned(), Value::String(value.to_owned()));
                }
                serde_json::to_vec_pretty(&config).unwrap()
            }
            _ => bytes,
        };
        writer
            .start_file(
                name,
                zip::write::SimpleFileOptions::default().compression_method(compression),
            )
            .unwrap();
        writer.write_all(&content).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn verify_witness(prefix: &str, project: &[u8], expected_markers: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("{prefix}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = root.join("input.3mf");
    fs::write(&input_path, project).unwrap();
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    let args = [
        "--slice".to_owned(),
        "1".to_owned(),
        "--outputdir".to_owned(),
        native_dir.to_str().unwrap().to_owned(),
        input_path.to_str().unwrap().to_owned(),
    ];
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let reference = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let reference_text = std::str::from_utf8(&reference).unwrap();
    for marker in expected_markers {
        assert!(
            reference_text.lines().any(|line| line == *marker),
            "Native output missing witness line `{marker}` in {root:?}"
        );
    }

    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_or_else(|error| panic!("API slice failed in {root:?}: {error:?}"));
    golden::compare_ordered_bytes_generator_only(&reference, &api)
        .unwrap_or_else(|error| panic!("API mismatch in {root:?}: {error}"));

    let cli_output = root.join("ares.gcode");
    let cli = crate::runner::application::CliCommand::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_output)
        .arg(&input_path)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "CLI exit {:?} in {root:?}",
        cli.status
    );
    let cli_bytes = fs::read(&cli_output).unwrap();
    golden::compare_ordered_bytes_generator_only(&reference, &cli_bytes)
        .unwrap_or_else(|error| panic!("CLI mismatch in {root:?}: {error}"));
    assert_eq!(
        api, cli_bytes,
        "raw API and CLI output must match in {root:?}"
    );
}
