//! Headless branch of `bbs_3mf.cpp:6418`, `_save_model_to_file`.
mod geometry;
mod project_settings;
mod xml;

use std::io::{Cursor, Write};

use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{LoadSettingsRequest, SliceError};

const MODEL: &str = "3D/3dmodel.model";
const SETTINGS: &str = "Metadata/project_settings.config";
const BAMBU_VERSION: &str = "02.08.01.55";

/// Export one ordinary mesh as a headless unsliced FFF project. The model and
/// presets are already in memory; the filename is only a model/source label.
pub fn export_3mf(
    input: &[u8],
    filename: &str,
    presets: LoadSettingsRequest<'_>,
) -> Result<Vec<u8>, SliceError> {
    if filename.is_empty() || filename.contains(['/', '\\', '\0']) {
        return Err(invalid(
            "export requires a source filename without a directory",
        ));
    }
    let mesh = geometry::load(input, filename)?;
    let settings = project_settings::write(&presets)?;
    let object_path = format!("3D/Objects/{}_1.model", mesh.name);
    let entries = [
        ("[Content_Types].xml", xml::content_types().into_bytes()),
        (MODEL, xml::main_model(&mesh, &object_path).into_bytes()),
        (
            "3D/_rels/3dmodel.model.rels",
            xml::relationships(&object_path).into_bytes(),
        ),
        (&object_path, xml::mesh_model(&mesh).into_bytes()),
        (SETTINGS, settings),
        (
            "Metadata/model_settings.config",
            xml::model_settings(&mesh, filename).into_bytes(),
        ),
        ("Metadata/slice_info.config", xml::slice_info().into_bytes()),
        (
            "Metadata/filament_sequence.json",
            b"{\"plate_1\":{\"nozzle_sequence\":[],\"optimal_assignment\":[],\"sequence\":[]}}"
                .to_vec(),
        ),
        ("_rels/.rels", xml::relationships(MODEL).into_bytes()),
    ];
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    // Native miniz supplies local wall time when absent; the ZIP epoch keeps
    // the envelope repeatable without changing a member payload.
    let options = SimpleFileOptions::default()
        .last_modified_time(zip::DateTime::default())
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, bytes) in entries {
        writer
            .start_file(name, options)
            .map_err(|err| invalid(err.to_string()))?;
        writer
            .write_all(&bytes)
            .map_err(|err| invalid(err.to_string()))?;
    }
    Ok(writer
        .finish()
        .map_err(|err| invalid(err.to_string()))?
        .into_inner())
}

fn invalid(reason: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(reason.into())
}

fn deferred(reason: &str) -> SliceError {
    invalid(format!("unsupported 3MF export: {reason}"))
}
