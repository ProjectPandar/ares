use std::fs;

use ares_core::{GenerationMetadata, SlicerContext, slice_project};
use serde_json::json;
use sha2::{Digest as _, Sha256};

#[path = "gcode_thumbnail_images/project.rs"]
mod project;

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[tokio::test]
async fn image_free_public_project_exports_preserve_complete_streams_and_load_errors() {
    let work = tempfile::Builder::new()
        .prefix("gcode-image-free-")
        .tempdir()
        .unwrap()
        .keep();
    let cases = [
        ("png", project::ENDER3, "32x24/PNG", false),
        ("jpg", project::ENDER3, "32x24/JPG", false),
        ("qoi", project::ENDER3, "32x24/QOI", false),
        ("btt", project::ENDER3, "32x24/BTT_TFT", false),
        ("colpic", project::ENDER3, "32x24/COLPIC", false),
        (
            "mixed",
            project::ENDER3,
            "32x24/PNG, 64x48/JPG, 32x24/PNG",
            false,
        ),
        ("empty", project::ENDER3, "", false),
        ("bbl-png", project::BBL, "32x24/PNG", false),
        ("bbl-btt", project::BBL, "32x24/BTT_TFT", false),
        ("invalid", project::ENDER3, "bad, 1000x24/WRONG", true),
        ("bbl-invalid", project::BBL, "bad, 1000x24/WRONG", true),
    ];
    let mut receipts = Vec::new();
    for (name, fixture, definitions, expected_error) in cases {
        let input = project::make(fixture, definitions);
        fs::write(work.join(format!("{name}.3mf")), &input).unwrap();
        let first =
            slice_project(&input, GenerationMetadata::default(), SlicerContext::new(0)).await;
        let second =
            slice_project(&input, GenerationMetadata::default(), SlicerContext::new(0)).await;
        let outcome = match (first, second) {
            (Ok(first), Ok(second)) => {
                assert!(
                    !expected_error,
                    "original load error unexpectedly disappeared: {name}"
                );
                fs::write(work.join(format!("{name}.gcode")), &first).unwrap();
                fs::write(work.join(format!("{name}.repeat.gcode")), &second).unwrap();
                assert_eq!(
                    first, second,
                    "image-free stream is not repeatable: {work:?}/{name}"
                );
                json!({"kind":"complete-stream", "bytes":first.len(), "sha256":hash(&first)})
            }
            (Err(first), Err(second)) => {
                assert!(
                    expected_error,
                    "unexpected original errors for {name}: {first}; {second}"
                );
                let first = first.to_string();
                let second = second.to_string();
                fs::write(work.join(format!("{name}.error.txt")), &first).unwrap();
                fs::write(work.join(format!("{name}.repeat.error.txt")), &second).unwrap();
                assert_eq!(
                    first, second,
                    "image-free load error changed: {work:?}/{name}"
                );
                json!({"kind":"existing-load-error", "message":first, "sha256":hash(first.as_bytes())})
            }
            (Ok(_), Err(error)) => panic!("repeat unexpectedly failed for {name}: {error}"),
            (Err(error), Ok(_)) => panic!("first export unexpectedly failed for {name}: {error}"),
        };
        receipts.push(json!({
            "name":name, "thumbnails":definitions, "input_sha256":hash(&input), "outcome":outcome,
            "comparison":"Ares original image-free result, identical metadata and seed 0"
        }));
        fs::write(
            work.join("result.json"),
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .unwrap();
    }
    eprintln!("original image-free results: {}", work.display());
}
