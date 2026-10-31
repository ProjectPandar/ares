use super::ExportedCase;
use serde_json::json;
use std::io::{Cursor, Read, Write};

// Failure inventory: preset export keeps two diameters but only one palette/map
// entry; a T1 template needs a fully bound two-logical-filament input before
// either slicer runs. Never mutate the shared Native export cache.
pub(crate) fn bind_two_filaments(mut exported: ExportedCase) -> ExportedCase {
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(settings["filament_diameter"].as_array().unwrap().len(), 2);
            settings["filament_map"] = json!(["1", "1"]);
            settings["filament_colour"] = json!(["#F2754E", "#0000FF"]);
            settings["flush_volumes_matrix"] = json!(["0", "140", "140", "0"]);
            settings["flush_volumes_vector"] = json!(["140", "140", "140", "140"]);
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
    let output_dir = tempfile::Builder::new()
        .prefix("vg1_flush_state-two-filament-")
        .tempdir()
        .unwrap()
        .keep();
    let project_path = output_dir.join("two-filament-input.3mf");
    std::fs::write(&project_path, &project).unwrap();
    exported.project = project;
    exported.project_path = project_path;
    exported.output_dir = output_dir;
    exported
}
