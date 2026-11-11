//! SeamPlacer candidate scoring and sharp-corner placement on a rotated object.

use std::{fs, path::Path};

#[tokio::test]
async fn rotated_object_first_layer_aligned_seam_matches_native_output() {
    let project = include_bytes!(
        "../../../../tests/parity/aligned-seam-rotated-object/rotated-two-cubes.3mf"
    );
    assert_eq!(
        crate::runner::native_api_cli::sha(project),
        "350250ac3b6f54b8ef79949d3232980bb4cd2a7b3bdbae74f71b3b62aa9faf79",
    );
    let runner = crate::OrcaRunner::from_env().unwrap().unwrap();
    let root = tempfile::Builder::new()
        .prefix("aligned-seam-rotated-object-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let native = native_plate(&runner, project, &root.join("reference"));
    let replay = native_plate(&runner, project, &root.join("replay"));
    let normalized = |gcode: &str| {
        crate::golden::normalize_one_generator_line(
            gcode.as_bytes(),
            crate::golden::GeneratorKind::Orca,
        )
        .unwrap()
    };
    let stability = crate::golden::first_difference(&normalized(&native), &normalized(&replay))
        .map(|difference| difference.to_string());
    fs::write(
        root.join("native-replay.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "whole_export_stable_generator_only": stability.is_none(),
            "first_difference": stability, "plate": 1,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(stability.is_none(), "Native replay is unstable: {root:?}");

    let api = crate::runner::application::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 11, 1, 0, 0, 0),
        Some(1),
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    let api = std::str::from_utf8(&api).unwrap();
    fs::write(root.join("native.first-layer.gcode"), first_layer(&native)).unwrap();
    fs::write(root.join("ares.first-layer.gcode"), first_layer(api)).unwrap();
    let expected = rotated_object(&native);
    let actual = rotated_object(api);
    fs::write(root.join("native.rotated-object.gcode"), expected).unwrap();
    fs::write(root.join("ares.rotated-object.gcode"), actual).unwrap();
    let difference = crate::golden::first_difference(expected.as_bytes(), actual.as_bytes())
        .map(|difference| difference.to_string());
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "boundary": "entire first-layer rotated-object block, including labels and terminal",
            "native_bytes": expected.len(), "api_bytes": actual.len(),
            "first_difference": difference, "records_filtered": 0,
            "whole_export": crate::golden::compare_ordered_bytes_generator_only(native.as_bytes(), api.as_bytes()),
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(actual, expected, "aligned-seam output differs: {root:?}");
}

fn native_plate(runner: &crate::OrcaRunner, project: &[u8], root: &Path) -> String {
    fs::create_dir(root).unwrap();
    let input = root.join("input.3mf");
    let output = root.join("output");
    fs::write(&input, project).unwrap();
    fs::create_dir(&output).unwrap();
    crate::runner::stages::run_live(
        runner.bin(),
        [
            "--slice".to_owned(),
            "1".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            input.to_str().unwrap().to_owned(),
        ],
        crate::runner::stages::Stage::Slice,
        root,
    )
    .unwrap();
    fs::read_to_string(output.join("plate_1.gcode")).unwrap()
}

fn first_layer(gcode: &str) -> &str {
    let start = gcode.find(";LAYER_CHANGE\n;Z:0.2\n").unwrap();
    let end = gcode[start + 1..].find(";LAYER_CHANGE\n").unwrap() + start + 1;
    &gcode[start..end]
}

fn rotated_object(gcode: &str) -> &str {
    let layer = first_layer(gcode);
    let start = layer
        .find("; printing object cube10.stl id:1 copy 0\n")
        .unwrap();
    let terminal = "; stop printing object cube10.stl id:1 copy 0\n";
    let end = layer[start..].find(terminal).unwrap() + start + terminal.len();
    &layer[start..end]
}
