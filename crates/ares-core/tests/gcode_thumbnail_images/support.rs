use std::{fs, process::Command};

#[path = "../../../../tests/support/slice_project.rs"]
mod direct_project;
use ares_core::{GenerationMetadata, SlicerContext, ThumbnailData};
use direct_project::slice_project;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};

use super::{
    project,
    samples::{self, Frame},
};

#[path = "../../../../tests/support/native_environment.rs"]
mod native_environment;

pub enum Expected {
    Blocks,
    Empty,
}

pub enum Control {
    CompleteStream,
    ExistingLoadError,
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub async fn codec(format: &str) {
    for (sample, frame) in samples::families() {
        let definitions = format!("{}x{}/{format}", frame.size[0], frame.size[1]);
        check(sample, &definitions, vec![frame], Expected::Blocks).await;
    }
}

pub async fn check(sample: &str, definitions: &str, frames: Vec<Frame>, expected: Expected) {
    let Some(binary) = native_environment::binary("ARES_GCODE_THUMBNAILS_BIN") else {
        return;
    };
    let work = tempfile::Builder::new()
        .prefix("gcode-thumbnail-images-")
        .tempdir()
        .unwrap()
        .keep();
    let input = project::make(project::ENDER3, definitions);
    fs::write(work.join("input.3mf"), &input).unwrap();
    let frame_requests: Vec<_> = frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            let rgba = work.join(format!("{index}.rgba"));
            fs::write(&rgba, &frame.pixels).unwrap();
            json!({"size": frame.size, "width": frame.width, "height": frame.height,
            "rgba": rgba, "rgba_sha256": hash(&frame.pixels)})
        })
        .collect();
    let request = work.join("request.json");
    fs::write(
        &request,
        serde_json::to_vec_pretty(&json!({
            "sample": sample, "thumbnails": definitions, "plate_id": 0,
            "input_sha256": hash(&input), "frames": frame_requests
        }))
        .unwrap(),
    )
    .unwrap();
    let native_path = work.join("native-blocks.gcode");
    let trace_path = work.join("native-result.json");
    fs::write(
        work.join("native-command.json"),
        serde_json::to_vec_pretty(&json!({
            "binary": binary, "arguments": [&request, &native_path, &trace_path]
        }))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(&binary)
        .args([&request, &native_path, &trace_path])
        .current_dir(&work)
        .output()
        .unwrap();
    fs::write(work.join("native-stdout.bin"), &output.stdout).unwrap();
    fs::write(work.join("native-stderr.bin"), &output.stderr).unwrap();
    fs::write(work.join("native-exit.txt"), output.status.to_string()).unwrap();
    assert!(output.status.success(), "Native observer failed: {work:?}");
    let native = fs::read(&native_path).unwrap();
    let trace: Value = serde_json::from_slice(&fs::read(&trace_path).unwrap()).unwrap();
    assert_eq!(trace["bytes"], json!(native.len()));
    for call in trace["calls"].as_array().unwrap() {
        assert_eq!(call["plate_id"], json!(0));
        for flag in [
            "printable_only",
            "parts_only",
            "show_bed",
            "transparent_background",
            "use_plate_box",
        ] {
            assert_eq!(call[flag], json!(true), "unexpected callback: {trace}");
        }
    }
    let images = frames
        .into_iter()
        .map(|frame| ThumbnailData::new(frame.width, frame.height, frame.pixels).unwrap())
        .collect();
    let result = slice_project(
        &input,
        GenerationMetadata::default(),
        SlicerContext::new(0).with_thumbnail_images(images),
    )
    .await;
    match expected {
        Expected::Blocks => {
            assert!(
                trace.get("error").is_none(),
                "Native rejected the input: {trace}"
            );
            assert!(!native.is_empty(), "Native emitted no blocks: {work:?}");
        }
        Expected::Empty => {
            assert!(
                trace.get("error").is_none(),
                "Native rejected the input: {trace}"
            );
            assert!(
                native.is_empty(),
                "Native unexpectedly emitted blocks: {work:?}"
            );
        }
    }
    let actual = result.unwrap();
    fs::write(work.join("ares.gcode"), &actual).unwrap();
    let baseline = slice_project(&input, GenerationMetadata::default(), SlicerContext::new(0))
        .await
        .unwrap();
    fs::write(work.join("image-free.gcode"), &baseline).unwrap();
    fs::write(
        work.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "sample": sample, "thumbnails": definitions, "input_sha256": hash(&input),
            "native_bytes": native.len(), "native_sha256": hash(&native),
            "ares_bytes": actual.len(), "ares_sha256": hash(&actual),
            "image_free_bytes": baseline.len(), "image_free_sha256": hash(&baseline),
            "comparison": "exact Native thumbnail bytes and complete remaining Ares stream"
        }))
        .unwrap(),
    )
    .unwrap();
    if native.is_empty() {
        assert_eq!(
            actual, baseline,
            "empty export changed complete stream: {work:?}"
        );
    } else {
        let matching: Vec<_> = actual
            .windows(native.len())
            .enumerate()
            .filter_map(|(index, bytes)| (bytes == native).then_some(index))
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "exact Native blocks missing or repeated: {work:?}"
        );
        let start = matching[0];
        let remaining = [&actual[..start], &actual[start + native.len()..]].concat();
        assert_eq!(
            remaining, baseline,
            "unrelated G-code bytes changed: {work:?}"
        );
    }
}

pub async fn unchanged(fixture: &[u8], definitions: &str, frames: Vec<Frame>, control: Control) {
    let work = tempfile::Builder::new()
        .prefix("gcode-thumbnail-control-")
        .tempdir()
        .unwrap()
        .keep();
    let input = project::make(fixture, definitions);
    fs::write(work.join("input.3mf"), &input).unwrap();
    let baseline =
        slice_project(&input, GenerationMetadata::default(), SlicerContext::new(0)).await;
    let images = frames
        .into_iter()
        .map(|frame| ThumbnailData::new(frame.width, frame.height, frame.pixels).unwrap())
        .collect();
    let actual = slice_project(
        &input,
        GenerationMetadata::default(),
        SlicerContext::new(0).with_thumbnail_images(images),
    )
    .await;
    let outcome = match control {
        Control::CompleteStream => {
            let baseline = baseline.unwrap();
            let actual = actual.unwrap();
            fs::write(work.join("image-free.gcode"), &baseline).unwrap();
            fs::write(work.join("ares.gcode"), &actual).unwrap();
            assert_eq!(
                actual, baseline,
                "complete no-export control changed: {work:?}"
            );
            json!({"kind":"complete-stream", "sha256":hash(&actual)})
        }
        Control::ExistingLoadError => {
            let baseline = baseline.unwrap_err().to_string();
            let actual = actual.unwrap_err().to_string();
            fs::write(work.join("image-free.error.txt"), &baseline).unwrap();
            fs::write(work.join("ares.error.txt"), &actual).unwrap();
            assert_eq!(actual, baseline, "original load error changed: {work:?}");
            json!({"kind":"existing-load-error", "message":actual})
        }
    };
    fs::write(
        work.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "thumbnails":definitions, "input_sha256":hash(&input), "outcome":outcome,
            "comparison":"original Ares image-free result, not a Native image oracle"
        }))
        .unwrap(),
    )
    .unwrap();
}
