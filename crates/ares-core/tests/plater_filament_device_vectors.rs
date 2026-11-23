use std::io::{Cursor, Read, Write};

use ares_core::{
    ConfigOverrides, FilamentSlotEdit, GenerationMetadata, Placement, PlaterSession,
    ProfileSliceRequest, SlicerContext, slice_plater_with_profiles,
};
use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const MACHINE: &str = "Bambu Lab X1 Carbon 0.4 nozzle";
const PROCESS: &str = "0.20mm Standard @BBL X1C";
const DEVICE_KEY: &str = "filament_dev_ams_drying_temperature";

fn project() -> Vec<u8> {
    let source = include_bytes!("../../../tests/parity/type1-compacted-exit/x1c-two-slots.3mf");
    let mut archive = ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() != "Metadata/project_settings.config" {
            output.raw_copy_file(entry).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        let mut config: Value = serde_json::from_slice(&bytes).unwrap();
        config["filament_settings_id"] = json!(["First PLA", "Second PLA"]);
        config["inherits_group"] = json!(["", "", "", ""]);
        config["enable_prime_tower"] = json!("0");
        output
            .start_file(entry.name(), SimpleFileOptions::default())
            .unwrap();
        output
            .write_all(&serde_json::to_vec(&config).unwrap())
            .unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn catalog() -> String {
    json!({"vendors": [{
        "id": "Acme",
        "index": {
            "name": "Acme",
            "machine_list": [{"sub_path": "printer.json"}],
            "process_list": [{"sub_path": "process.json"}],
            "filament_list": [{"sub_path": "first.json"}, {"sub_path": "second.json"}]
        },
        "files": {
            "printer.json": {
                "type": "machine", "name": MACHINE, "from": "system", "instantiation": "true",
                "printer_model": "Bambu Lab X1 Carbon", "printer_variant": "0.4",
                "single_extruder_multi_material": "1", "nozzle_diameter": [0.4],
                "printable_area": ["0x0", "256x0", "256x256", "0x256"]
            },
            "process.json": {
                "type": "process", "name": PROCESS, "from": "system", "instantiation": "true",
                "enable_prime_tower": "0"
            },
            "first.json": {
                "type": "filament", "name": "First PLA", "from": "system", "instantiation": "true",
                "filament_dev_ams_drying_temperature": [35, 37]
            },
            "second.json": {
                "type": "filament", "name": "Second PLA", "from": "system", "instantiation": "true",
                "filament_dev_ams_drying_temperature": [41, 43, 47]
            }
        }
    }], "imports": []})
    .to_string()
}

async fn emitted_temperatures(session: &PlaterSession, request: ProfileSliceRequest<'_>) -> String {
    let output = slice_plater_with_profiles(
        session,
        request,
        false,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let marker = format!("; {DEVICE_KEY} = ");
    std::str::from_utf8(&output.gcode)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix(&marker))
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn editing_physical_slots_keeps_whole_device_vectors_with_their_presets() {
    let catalog = catalog();
    let request = ProfileSliceRequest {
        filename: "two-materials.3mf",
        catalog_json: &catalog,
        selection_json: r#"{"from_project":true,"load_and_select":"always"}"#,
        plate: None,
        overrides: &ConfigOverrides::default(),
        placement: Placement::None,
    };
    let mut session = PlaterSession::load_with_profiles(project(), request)
        .await
        .unwrap();
    assert_eq!(
        emitted_temperatures(&session, request).await,
        "35,37,41,43,47"
    );
    session.edit_filament_slots(FilamentSlotEdit::Add).unwrap();
    assert_eq!(
        emitted_temperatures(&session, request).await,
        "35,37,41,43,47,41,43,47"
    );
    session
        .edit_filament_slots(FilamentSlotEdit::Delete { slot: 1 })
        .unwrap();
    assert_eq!(
        emitted_temperatures(&session, request).await,
        "41,43,47,41,43,47"
    );
}
