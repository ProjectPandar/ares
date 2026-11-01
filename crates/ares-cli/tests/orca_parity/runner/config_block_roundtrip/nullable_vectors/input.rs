use serde_json::{Map, Value};
use std::io::{Cursor, Write};

pub(super) fn settings_variant(source: &[u8], updates: &Map<String, Value>) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut settings = super::super::super::application::settings(source).unwrap();
    let original = settings.clone();
    settings.extend(updates.clone());
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    let result = writer.finish().unwrap().into_inner();
    let mut actual = super::super::super::application::settings(&result).unwrap();
    let mut expected = original;
    for (key, value) in updates {
        assert_eq!(actual.remove(key).as_ref(), Some(value));
        expected.remove(key);
    }
    assert_eq!(actual, expected);
    let mut after = zip::ZipArchive::new(Cursor::new(&result)).unwrap();
    assert_eq!(archive.len(), after.len());
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut corresponding = after.by_name(entry.name()).unwrap();
        let mut before_bytes = Vec::new();
        let mut after_bytes = Vec::new();
        std::io::copy(&mut entry, &mut before_bytes).unwrap();
        std::io::copy(&mut corresponding, &mut after_bytes).unwrap();
        assert_eq!(before_bytes, after_bytes, "{}", entry.name());
    }
    result
}
