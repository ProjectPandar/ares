use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

const BASE: &[u8] =
    include_bytes!("../../../../../tests/parity/wipe-before-override/control-before50.3mf");

pub(super) fn project(after: &str, override_after: &str, wipe: &str) -> Vec<u8> {
    assert_eq!(
        crate::runner::native_api_cli::sha(BASE),
        "adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2"
    );
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(settings["retract_before_wipe"], json!(["50%"]));
            assert_eq!(settings["retraction_length"], json!(["0.8"]));
            assert_eq!(settings["wipe_distance"], json!(["2"]));
            assert_eq!(settings["wipe"], json!(["1"]));
            assert!(settings.get("retract_after_wipe").is_none());
            assert!(settings.get("filament_retract_after_wipe").is_none());
            settings["retract_after_wipe"] = json!([after]);
            settings["filament_retract_after_wipe"] = json!([override_after]);
            settings["wipe"] = json!([wipe]);
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
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
    let mut result = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    assert_eq!(reader.len(), result.len());
    for index in 0..reader.len() {
        let mut before = reader.by_index(index).unwrap();
        let mut after = result.by_index(index).unwrap();
        assert_eq!(before.name(), after.name());
        let mut a = Vec::new();
        let mut b = Vec::new();
        before.read_to_end(&mut a).unwrap();
        after.read_to_end(&mut b).unwrap();
        if before.name() == "Metadata/project_settings.config" {
            let original: Value = serde_json::from_slice(&a).unwrap();
            let mut actual: Value = serde_json::from_slice(&b).unwrap();
            let map = actual.as_object_mut().unwrap();
            map.remove("retract_after_wipe");
            map.remove("filament_retract_after_wipe");
            map.insert("wipe".into(), json!(["1"]));
            assert_eq!(original, actual);
        } else {
            assert_eq!(a, b, "{}", before.name());
        }
    }
    bytes
}
