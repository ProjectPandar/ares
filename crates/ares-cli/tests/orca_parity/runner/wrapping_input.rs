//! Preserve Native-exported placement; bind only the detection flag for Print validation.
use super::{ExportedCase, application};
use serde_json::json;
use std::{
    io::{Cursor, Read, Write},
    path::Path,
};

pub(crate) fn enable(mut exported: ExportedCase, root: &Path) -> ExportedCase {
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(settings["enable_wrapping_detection"], json!("0"));
            settings["enable_wrapping_detection"] = json!("1");
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let project = output.finish().unwrap().into_inner();
    let mut actual = application::settings(&project).unwrap();
    assert_eq!(actual.remove("enable_wrapping_detection"), Some(json!("1")));
    let mut original = application::settings(&exported.project).unwrap();
    assert_eq!(
        original.remove("enable_wrapping_detection"),
        Some(json!("0"))
    );
    assert_eq!(actual, original);
    let mut rewritten = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    assert_eq!(input.len(), rewritten.len());
    for index in 0..input.len() {
        let mut before = input.by_index(index).unwrap();
        if before.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut after = rewritten.by_name(before.name()).unwrap();
        let mut a = Vec::new();
        let mut b = Vec::new();
        before.read_to_end(&mut a).unwrap();
        after.read_to_end(&mut b).unwrap();
        assert_eq!(a, b, "{}", before.name());
    }
    let output_dir = root.join("wrapping-fixed-placement");
    std::fs::create_dir_all(&output_dir).unwrap();
    let project_path = output_dir.join("wrapping-input.3mf");
    std::fs::write(&project_path, &project).unwrap();
    exported.project = project;
    exported.project_path = project_path;
    exported.output_dir = output_dir;
    exported.label = "bounded-wrapping/fixed-placement-enabled".into();
    exported
}
