//! GCode.cpp:3894-3908,3945-3950; Print.cpp:4488-4492,4543-4552; Print.hpp:1294: the start
//! template reads the first-layer bounds in print space, after the plate origin is subtracted
//! from the world-frame hull.
use std::{
    fs,
    io::{Cursor, Read, Write},
};

use crate::runner::application::slice_direct_project_plate as slice_project_plate;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};

use crate::runner::{ExportedCase, OrcaRunner, application, cache_identity, repo_root};

const SOURCE: &str = "tests/parity/validation/print-bed-two-plates.3mf";
const SPARSE_INFILL_TWO: &str = r#"key="sparse_infill_filament_id" value="2""#;
const SPARSE_INFILL_ONE: &str = r#"key="sparse_infill_filament_id" value="1""#;

/// `SOURCE` with its object's sparse infill on the first filament, so the plate uses one
/// filament and neither side prints a wipe tower.
fn one_filament(source: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/model_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert!(
                text.contains(SPARSE_INFILL_TWO),
                "object no longer uses filament 2"
            );
            bytes = text
                .replace(SPARSE_INFILL_TWO, SPARSE_INFILL_ONE)
                .into_bytes();
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

/// X of the build item's world transform in `3D/3dmodel.model`.
fn build_item_x(source: &[u8]) -> f64 {
    let mut input = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut model = String::new();
    input
        .by_name("3D/3dmodel.model")
        .unwrap()
        .read_to_string(&mut model)
        .unwrap();
    let item = &model[model.find("<item ").unwrap()..];
    let transform = item.split("transform=\"").nth(1).unwrap();
    let values = transform[..transform.find('"').unwrap()]
        .split_whitespace()
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    values[9]
}

/// The executable block up to the end of the machine start template, without the `M73`
/// progress lines; `length` caps it at Native's region.
fn start_region(gcode: &str, length: Option<usize>) -> Vec<&str> {
    let lines = gcode
        .lines()
        .skip_while(|line| *line != "; EXECUTABLE_BLOCK_START")
        .filter(|line| !line.starts_with("M73 "));
    match length {
        Some(length) => lines.take(length).collect(),
        None => lines
            .take_while(|line| *line != "; MACHINE_START_GCODE_END")
            .collect(),
    }
}

#[tokio::test]
async fn later_plate_first_layer_bounds_render_in_print_space() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let source = fs::read(repo_root().join(SOURCE)).unwrap();
    let settings = application::settings(&source).unwrap();
    let bed_width = settings["printable_area"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            let (x, _) = point.as_str().unwrap().split_once('x').unwrap();
            x.parse::<f64>().unwrap()
        })
        .fold(f64::MIN, f64::max);
    let item_x = build_item_x(&source);
    assert!(
        item_x > bed_width,
        "the object at {item_x} is on the first plate"
    );
    assert!(
        settings["machine_start_gcode"]
            .as_str()
            .unwrap()
            .contains("first_layer_print_min"),
        "the start template does not read the first-layer bounds"
    );

    let project = one_filament(&source);
    let project_path = runner.work.join("print-bed-two-plates-one-filament.3mf");
    fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_selected_plate_case(
            &ExportedCase {
                label: "validation/first-layer-print-space/plate-2".to_owned(),
                project: project.clone(),
                project_path,
                output_dir,
            },
            2,
        )
        .unwrap();
    let gcode = slice_project_plate(
        project,
        GenerationMetadata::default(),
        Some(2),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(runner.work.join("ares.gcode"), &gcode).unwrap();
    fs::write(runner.work.join("orca.gcode"), &case.reference).unwrap();

    let native = String::from_utf8(case.reference.clone()).unwrap();
    let ares = String::from_utf8(gcode.clone()).unwrap();
    let expected = start_region(&native, None);
    assert!(
        expected
            .iter()
            .any(|line| line.trim_start().starts_with("G29 A1 ")),
        "Native rendered no bed-mesh area"
    );
    let actual = start_region(&ares, Some(expected.len()));
    fs::write(
        runner.work.join("start-region-comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "fixture": SOURCE,
            "plate": 2,
            "input_sha256": cache_identity::raw_digest(&case.project),
            "native_region_lines": expected.len(),
            "ares_region_lines": actual.len(),
            "whole_output_parity_claimed": false,
            "artifacts": runner.work,
        }))
        .unwrap(),
    )
    .unwrap();
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_eq!(
            actual,
            expected,
            "start region line {} at {:?}",
            index + 1,
            runner.work
        );
    }
    assert_eq!(actual.len(), expected.len(), "at {:?}", runner.work);
}
