use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

pub(super) const DEFAULT: &str = "120 100 6.6 6.8 7.2 7.6 7.9 8.2 8.7 9.4 9.9 10.0| 0.05 6.6 0.45 6.8 0.95 7.8 1.45 8.3 1.95 9.7 2.45 10 2.95 7.6 3.45 7.6 3.95 7.6 4.45 7.6 4.95 7.6";
pub(super) const SHORT: &str = "100 100 1 1|0.05 1 1";
pub(super) const WIDTH: &str = "120 100 1 1|0.05 1 1";
const BASE: &[u8] = include_bytes!("../../../../../../tests/parity/semm-ramming/R1.3mf");

pub(super) fn project(profiles: &[&str], gate: &str) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut original = None;
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let before: Value = serde_json::from_slice(&bytes).unwrap();
            for (key, expected) in [
                ("filament_ramming_parameters", json!([DEFAULT, SHORT])),
                ("enable_filament_ramming", json!("1")),
                ("single_extruder_multi_material", json!("1")),
                ("enable_prime_tower", json!("1")),
                ("purge_in_prime_tower", json!("1")),
                ("filament_map", json!(["1", "1"])),
                ("filament_is_support", json!(["0", "0"])),
                ("filament_colour", json!(["#F2754E", "#0000FF"])),
                ("nozzle_diameter", json!(["0.4"])),
                ("wipe_tower_x", json!(["150"])),
                ("wipe_tower_y", json!(["150"])),
            ] {
                assert_eq!(before[key], expected, "{key}");
            }
            let mut after = before.clone();
            after["filament_ramming_parameters"] = json!(profiles);
            after["enable_filament_ramming"] = json!(gate);
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
            assert_eq!(actual["filament_ramming_parameters"], json!(profiles));
            assert_eq!(actual["enable_filament_ramming"], json!(gate));
            actual["filament_ramming_parameters"] = json!([DEFAULT, SHORT]);
            actual["enable_filament_ramming"] = json!("1");
            assert_eq!(actual, original.as_ref().unwrap().clone());
        } else {
            assert_eq!(aa, bb, "{}", a.name());
        }
    }
    bytes
}
