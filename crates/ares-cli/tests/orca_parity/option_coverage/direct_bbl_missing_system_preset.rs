//! CLI:2805–2807 logs a missing system filament and preserves its slot.

use crate::{OrcaRunner, runner::native_api_cli::NativeCase};
use std::io::{Cursor, Read, Write};

#[test]
fn direct_bbl_missing_system_preset_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = missing_preset_project();
    let case = NativeCase::slice(&runner, &project, "direct-bbl-missing-system-preset-");
    for setting in [
        "filament_settings_id = "Missing system filament fixture"",
        "filament_density = 1.24",
        "filament_flow_ratio = 0.98",
        "filament_diameter = 1.75",
    ] {
        assert!(case.native.contains(&format!("\n; {setting}\n")));
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}

fn missing_preset_project() -> Vec<u8> {
    let base = include_bytes!("../../../../../tests/parity/orca_cli_ender3.3mf");
    let mut archive = zip::ZipArchive::new(Cursor::new(base)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .last_modified_time(zip::DateTime::default());
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            raw["filament_settings_id"] = serde_json::json!(["Missing system filament fixture"]);
            bytes = serde_json::to_vec(&raw).unwrap();
        }
        output.start_file(file.name(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}
