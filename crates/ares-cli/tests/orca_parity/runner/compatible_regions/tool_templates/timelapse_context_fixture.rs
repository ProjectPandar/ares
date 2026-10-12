use crate::{
    self as parity,
    runner::{ExportedCase, OrcaRunner, application},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};

use super::super::layer_context::fixture as admitted;

// Failure inventory: preserve the admitted two-tool project, reset templates
// and non-settings ZIP members; require the actual loaded timelapse string.
pub(super) fn prepare(
    runner: &OrcaRunner,
    label: &str,
    first: &[usize],
    script: Option<&str>,
    handoff: bool,
) -> ExportedCase {
    let mut exported = admitted::prepare(runner, label, first, &[], handoff);
    let before = application::settings(&exported.project).unwrap();
    let original = before["time_lapse_gcode"].as_str().unwrap();
    assert!(original.is_empty(), "vendor timelapse admission");
    let requested = script.unwrap_or(original);
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut hashes = serde_json::Map::new();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["time_lapse_gcode"] = json!(requested);
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        } else {
            hashes.insert(
                file.name().into(),
                json!(
                    Sha256::digest(&bytes)
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                ),
            );
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    exported.project = output.finish().unwrap().into_inner();
    let mut actual = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    for (name, hash) in &hashes {
        let mut bytes = Vec::new();
        actual
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(
            *hash,
            json!(
                Sha256::digest(&bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
            "nonowned member {name}"
        );
    }
    let effective =
        application::verify_value(&exported.project, "time_lapse_gcode", &json!(requested))
            .unwrap();
    let loaded = ares_core::load_project(&exported.project).unwrap();
    assert_eq!(
        loaded.settings().printer.gcode.time_lapse_gcode.0,
        requested
    );
    let root = parity::artifacts::root_from_env().unwrap();
    let stem = label.replace('/', "-");
    parity::artifacts::overwrite(&exported.project_path, &exported.project).unwrap();
    parity::artifacts::overwrite(&root.join(format!("{stem}-input.3mf")), &exported.project)
        .unwrap();
    parity::artifacts::overwrite(
        &root.join(format!("{stem}-timelapse-application.json")),
        &serde_json::to_vec_pretty(&json!({
            "source":"GCode.cpp:4672-4674,5167", "requested":requested,
            "vendor":original, "effective":effective, "nonowned_member_sha256":hashes,
            "typed":loaded.settings().printer.gcode.time_lapse_gcode.0,
        }))
        .unwrap(),
    )
    .unwrap();
    exported
}
