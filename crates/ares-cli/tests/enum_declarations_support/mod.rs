use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn artifacts(name: &str) -> PathBuf {
    let root = std::env::var_os("ARES_ENUM_ARTIFACTS")
        .expect("set ARES_ENUM_ARTIFACTS to a fresh directory");
    let path = Path::new(&root).join(name);
    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn loaded_value(project: &ares_core::Project, key: &str) -> Value {
    let settings = project.settings();
    match key {
        "top_surface_pattern" => json!(settings.process.region.top_surface_pattern),
        "bottom_surface_pattern" => json!(settings.process.region.bottom_surface_pattern),
        "internal_solid_infill_pattern" => {
            json!(settings.process.region.internal_solid_infill_pattern)
        }
        "support_interface_pattern" => json!(settings.process.object.support_interface_pattern),
        "brim_type" => json!(settings.process.object.brim_type),
        "print_order" => json!(settings.process.print.print_order),
        "host_type" => json!(settings.printer.remaining.host_type),
        "filament_map_mode" => json!(settings.project.gcode.filament_map_mode),
        _ => panic!("unregistered test key: {key}"),
    }
}

pub fn input(key: &str, value: &str) -> Vec<u8> {
    let bytes = std::fs::read(repo().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let mut source = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_reader(&mut entry).unwrap();
            settings[key] = json!(value);
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}
