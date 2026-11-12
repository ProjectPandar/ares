use std::io::{Cursor, Read as _, Write as _};

use serde_json::{Value, json};

pub const ENDER3: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");
pub const BBL: &[u8] =
    include_bytes!("../../../../tests/parity/bbl-layer-object-labels/empty-template.3mf");

pub fn make(fixture: &[u8], thumbnails: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(fixture)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let options = zip::write::SimpleFileOptions::default();
        if entry.is_dir() {
            writer.add_directory(entry.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["thumbnails"] = json!(thumbnails);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        writer.start_file(entry.name(), options).unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
