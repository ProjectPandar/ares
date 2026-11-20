//! Headless branch of `bbs_3mf.cpp:6418`, `_save_model_to_file`.
mod geometry;
mod project_settings;
mod xml;
mod zip_records;

use std::io::{Cursor, Seek, Write};

use md5::{Digest, Md5};
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{LoadSettingsRequest, ProcessedGCode, SliceError};

const MODEL: &str = "3D/3dmodel.model";
const SETTINGS: &str = "Metadata/project_settings.config";
const BAMBU_VERSION: &str = "02.08.01.55";
const PLATE_GCODE: &str = "Metadata/plate_1.gcode";

pub(crate) fn write_project_settings_raw(
    raw: &crate::project::raw_settings::ProjectSettingsRaw,
) -> Result<Vec<u8>, SliceError> {
    project_settings::write_raw(raw, true)
}

struct SlicedMembers {
    plate_json: Vec<u8>,
    gcode: Vec<u8>,
    slice_info: Vec<u8>,
    filament_sequence: Vec<u8>,
    identify_id: u32,
    orca_version: Option<String>,
}

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
    write_package(&mesh, filename, settings, None)
}

pub fn export_sliced_3mf(
    input: &[u8],
    filename: &str,
    processed: ProcessedGCode,
) -> Result<Vec<u8>, SliceError> {
    if filename.is_empty() || filename.contains(['/', '\\', '\0']) {
        return Err(invalid(
            "export requires a source filename without a directory",
        ));
    }
    let archive = processed
        .archive
        .as_ref()
        .ok_or_else(|| invalid("sliced export requires captured plate data"))?;
    if !filename.to_ascii_lowercase().ends_with(".3mf") {
        return Err(deferred("sliced export requires a saved 3MF project"));
    }
    let saved = geometry::load_saved(input)?;
    let plate_json = xml::plate_json(archive);
    let slice_info = xml::slice_info(archive, &processed).into_bytes();
    let filament_sequence = xml::filament_sequence();
    write_package(
        &saved.mesh,
        &saved.source_file,
        archive.project_settings.clone(),
        Some(SlicedMembers {
            plate_json,
            gcode: processed.gcode,
            slice_info,
            filament_sequence,
            identify_id: archive.identify_id,
            orca_version: saved.orca_version,
        }),
    )
}

fn write_package(
    mesh: &geometry::Mesh,
    filename: &str,
    settings: Vec<u8>,
    sliced: Option<SlicedMembers>,
) -> Result<Vec<u8>, SliceError> {
    let object_path = format!("3D/Objects/{}_1.model", mesh.name);
    let sliced_output = sliced.is_some();
    let identify_id = sliced.as_ref().map(|sliced| sliced.identify_id);
    let model_settings =
        xml::model_settings(mesh, filename, sliced_output, identify_id).into_bytes();
    let mut entries = vec![("[Content_Types].xml", xml::content_types().into_bytes())];
    if let Some(sliced) = &sliced {
        entries.push(("Metadata/plate_1.json", sliced.plate_json.clone()));
    }
    entries.extend([
        (
            MODEL,
            xml::main_model(
                mesh,
                &object_path,
                sliced.as_ref().and_then(|s| s.orca_version.as_deref()),
            )
            .into_bytes(),
        ),
        (
            "3D/_rels/3dmodel.model.rels",
            xml::relationships(&object_path).into_bytes(),
        ),
        (&object_path, xml::mesh_model(mesh).into_bytes()),
        (SETTINGS, settings),
    ]);
    let (slice_info, filament_sequence) = if let Some(sliced) = sliced {
        let gcode = sliced.gcode;
        entries.extend([
            (
                "Metadata/plate_1.gcode.md5",
                format!("{:X}", Md5::digest(&gcode)).into_bytes(),
            ),
            (PLATE_GCODE, gcode),
            (
                "Metadata/_rels/model_settings.config.rels",
                xml::gcode_relationship().into_bytes(),
            ),
        ]);
        (sliced.slice_info, sliced.filament_sequence)
    } else {
        (
            xml::slice_info_header().into_bytes(),
            b"{\"plate_1\":{\"nozzle_sequence\":[],\"optimal_assignment\":[],\"sequence\":[]}}"
                .to_vec(),
        )
    };
    entries.extend([
        ("Metadata/model_settings.config", model_settings),
        ("Metadata/slice_info.config", slice_info),
        ("Metadata/filament_sequence.json", filament_sequence),
        ("_rels/.rels", xml::relationships(MODEL).into_bytes()),
    ]);
    if sliced_output {
        let stream = write_members(ZipWriter::new_stream(Vec::new()), entries)?;
        let mut bytes = stream.into_inner();
        zip_records::write_native_fields(&mut bytes)?;
        Ok(bytes)
    } else {
        write_members(ZipWriter::new(Cursor::new(Vec::new())), entries).map(Cursor::into_inner)
    }
}

fn write_members<W: Write + Seek>(
    mut writer: ZipWriter<W>,
    entries: Vec<(&str, Vec<u8>)>,
) -> Result<W, SliceError> {
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
    writer.finish().map_err(|err| invalid(err.to_string()))
}

fn invalid(reason: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(reason.into())
}

fn deferred(reason: &str) -> SliceError {
    invalid(format!("unsupported 3MF export: {reason}"))
}
