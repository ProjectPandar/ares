use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

const BASE: &[u8] =
    include_bytes!("../../../../../../tests/parity/printer-physical-identity/P0.3mf");

pub(super) fn project(ids: &[&str], physical: &[&str]) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut original = None;
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let before: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(before["printer_extruder_id"], json!(["1", "1", "2", "2"]));
            assert_eq!(before["physical_extruder_map"], json!(["0", "1"]));
            assert_eq!(before["filament_map"], json!(["1", "2"]));
            assert_eq!(before["filament_map_mode"], "Manual");
            assert_eq!(
                before["retract_length_toolchange"],
                json!(["1", "1", "2", "2"])
            );
            assert_eq!(before["filament_type"], json!(["PLA", "PLA"]));
            let mut after = before.clone();
            after["printer_extruder_id"] = json!(ids);
            after["physical_extruder_map"] = json!(physical);
            bytes = serde_json::to_vec_pretty(&after).unwrap();
            original = Some(before);
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let bytes = writer.finish().unwrap().into_inner();
    let mut after = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    assert_eq!(reader.len(), after.len());
    for index in 0..reader.len() {
        let mut a = reader.by_index(index).unwrap();
        let mut b = after.by_index(index).unwrap();
        assert_eq!(a.name(), b.name());
        let mut aa = Vec::new();
        let mut bb = Vec::new();
        a.read_to_end(&mut aa).unwrap();
        b.read_to_end(&mut bb).unwrap();
        if a.name() == "Metadata/project_settings.config" {
            let mut actual: Value = serde_json::from_slice(&bb).unwrap();
            assert_eq!(actual["printer_extruder_id"], json!(ids));
            assert_eq!(actual["physical_extruder_map"], json!(physical));
            actual["printer_extruder_id"] = json!(["1", "1", "2", "2"]);
            actual["physical_extruder_map"] = json!(["0", "1"]);
            assert_eq!(actual, original.as_ref().unwrap().clone());
        } else {
            assert_eq!(aa, bb, "{}", a.name());
        }
    }
    bytes
}
