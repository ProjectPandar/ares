use super::super::{ExportedCase, application};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};

pub(super) fn threshold_input(
    exported: ExportedCase,
    threshold: &str,
    directory: &Path,
) -> ExportedCase {
    let effective = application::settings(&exported.project).unwrap();
    if effective["small_perimeter_threshold"] == json!([threshold]) {
        return exported;
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["small_perimeter_threshold"] = json!([threshold]);
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
    let project_path = directory.join("threshold-exact.3mf");
    fs::write(&project_path, &project).unwrap();
    fs::write(directory.join("threshold-transport.json"), serde_json::to_vec_pretty(&json!({"requested":threshold,"exported":effective["small_perimeter_threshold"],"edited_member":"Metadata/project_settings.config","all_other_member_payloads":"unchanged","export_sha256":super::super::cache_identity::raw_digest(&exported.project),"input_sha256":super::super::cache_identity::raw_digest(&project)})).unwrap()).unwrap();
    ExportedCase {
        project,
        project_path,
        ..exported
    }
}
