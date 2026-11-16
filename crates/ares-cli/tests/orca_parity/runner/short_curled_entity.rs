//! `ExtrusionProcessor.hpp:388–425`: exhausted short-entity window samples.

use super::{ExportedCase, OrcaRunner, application, cache_identity};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[test]
fn short_curled_entity_matches_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("short curled entity: no optional OrcaRunner");
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("short-curled-entity-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = project();
    let path = directory.join("input.3mf");
    fs::write(&path, &input).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "curled/short-entity/whole-file".into(),
            project: input.clone(),
            project_path: path,
            output_dir,
        })
        .unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(application::slice_project(
            &input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &actual).unwrap();
    fs::write(directory.join("orca.gcode"), &case.reference).unwrap();
    let native = std::str::from_utf8(&case.reference).unwrap();
    assert_eq!(
        native
            .lines()
            .filter(|line| *line == ";LAYER_CHANGE")
            .count(),
        50
    );
    for marker in [
        "; enable_prime_tower = 0",
        "; enable_overhang_speed = 1,1",
        "; slowdown_for_curled_perimeters = 1,1",
        "G1 X169.764 Y169.784 E.00222\nG1 X169.709 Y169.784 E.0022\nM106 S204\nG1 F5880\nG1 X169.681 Y169.736 E.00222",
    ] {
        assert!(
            native.contains(marker),
            "Native short-entity witness absent: {marker}; {directory:?}"
        );
    }
    let comparison = crate::golden::compare_ordered_bytes_generator_only(&case.reference, &actual);
    let status = if comparison.is_ok() {
        "PASS"
    } else {
        "DIVERGENT"
    };
    let detail = comparison.as_ref().err().cloned().unwrap_or_default();
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "status": status,
            "detail": detail,
            "input_sha256": cache_identity::raw_digest(&input),
            "upstream": "ExtrusionProcessor.hpp:388-425",
            "comparison": "complete output; generated-by identity only"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(status, "PASS", "{detail}; {directory:?}");
}

fn project() -> Vec<u8> {
    let template = include_bytes!("../../../../../tests/parity/validation/tiq2-type2-enabled.3mf");
    assert_eq!(
        cache_identity::raw_digest(template),
        "0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942"
    );
    let mut source = zip::ZipArchive::new(Cursor::new(template)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut file = source.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["enable_prime_tower"] = json!("0");
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        if file.name() == "3D/Objects/overhang.stl_1.model" {
            bytes = mesh_model().into_bytes();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn mesh_model() -> String {
    use std::fmt::Write;
    let vertices = [
        [-5.0, -5.0, -7.5],
        [-5.0, 5.0, -7.5],
        [5.0, 5.0, -7.5],
        [5.0, -5.0, -7.5],
        [-5.0, -5.0, 2.5],
        [5.0, 5.0, -7.5],
        [-5.0, 5.0, 2.5],
        [5.0, -5.0, 2.5],
        [5.0, 5.0, 2.5],
    ];
    let triangles = [
        [0, 1, 2],
        [0, 2, 3],
        [4, 5, 6],
        [4, 7, 5],
        [0, 3, 7],
        [0, 7, 4],
        [1, 6, 8],
        [1, 8, 2],
        [0, 4, 6],
        [0, 6, 1],
        [3, 2, 8],
        [3, 8, 7],
    ];
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\"><resources><object id=\"1\" type=\"model\"><mesh><vertices>",
    );
    for [x, y, z] in vertices {
        write!(xml, "<vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>").unwrap();
    }
    xml.push_str("</vertices><triangles>");
    for [v1, v2, v3] in triangles {
        write!(xml, "<triangle v1=\"{v1}\" v2=\"{v2}\" v3=\"{v3}\"/>").unwrap();
    }
    xml.push_str("</triangles></mesh></object></resources><build/></model>");
    xml
}
