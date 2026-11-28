//! Headless branch of `bbs_3mf.cpp:6418`, `_save_model_to_file`.
mod geometry;
mod json_float;
mod project_settings;
mod saved_assets;
mod saved_presets;
mod scene;
mod sliced;
mod stl;
mod xml;
mod zip_records;

pub use saved_presets::CliUnsliced3mf;
pub use scene::export_project_3mf;
pub(crate) use scene::write as write_plater_project;
pub use sliced::{
    CliSavedSlicedProject, export_saved_sliced_3mf, export_sliced_3mf, print_object_id,
};
pub use stl::export_imported_stl_3mf;

use std::io::{Cursor, Seek, Write};

use md5::{Digest, Md5};
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{LoadSettingsRequest, SliceError};
pub(crate) use geometry::{SavedModel, placed_stl_model};
pub(crate) use project_settings::direct_stl_settings;
use saved_assets::{PREVIEW_REFERENCES, PlatePreviews, SMALL_PREVIEW};

const MODEL: &str = "3D/3dmodel.model";
const SETTINGS: &str = "Metadata/project_settings.config";
const BAMBU_VERSION: &str = "02.08.01.55";

pub(crate) fn write_project_settings_raw(
    raw: &crate::project::raw_settings::ProjectSettingsRaw,
    saved: bool,
) -> Result<Vec<u8>, SliceError> {
    project_settings::write_raw(raw, saved)
}

/// The G-code members of a sliced package. `skip_model` is `SaveStrategy::SkipModel`
/// (`bbs_3mf.cpp:6328`), set by the CLI `--min-save` (`OrcaSlicer.cpp:8402-8403`) and by
/// `Plater::send_gcode` (`Plater.cpp:21240`).
struct Sliced {
    members: Vec<SlicedMembers>,
    skip_model: bool,
    auxiliary: Vec<(String, Vec<u8>)>,
}

struct SlicedMembers {
    plate: u32,
    object_ids: Vec<(usize, u32)>,
    plate_json: Vec<u8>,
    gcode: Vec<u8>,
    slice_info: Vec<u8>,
    previews: Option<PlatePreviews>,
}

struct ObjectEntry<'a> {
    object: &'a geometry::Object,
    id: u32,
    backup_id: u32,
    path: String,
}

impl ObjectEntry<'_> {
    fn parts(&self) -> impl Iterator<Item = (u32, u32, &geometry::Part)> {
        let first = self.id - self.object.parts.len() as u32;
        self.object
            .parts
            .iter()
            .enumerate()
            .map(move |(index, part)| {
                let index = index as u32;
                (first + index, (self.backup_id << 16) + index, part)
            })
    }
}

/// Export one ordinary mesh or saved project as a headless unsliced FFF project. The model and
/// presets are already in memory; the filename is only a model/source label.
pub fn export_3mf(
    input: &[u8],
    filename: &str,
    presets: LoadSettingsRequest<'_>,
) -> Result<Vec<u8>, SliceError> {
    CliUnsliced3mf::admit(input, filename)?.export(presets)
}

pub fn export_arranged_sliced_3mf(
    mut plates: Vec<crate::SlicedStlPlate>,
) -> Result<Vec<u8>, SliceError> {
    let mut archives = Vec::with_capacity(plates.len());
    let mut model: Option<SavedModel> = None;
    for plate in &mut plates {
        let mut archive = plate
            .processed
            .archive
            .take()
            .ok_or_else(|| invalid("sliced export requires captured plate data"))?;
        if archive.filament_color.is_empty()
            && let Some(color) = plate.processed.result.extruder_colors.first()
        {
            archive.filament_color.clone_from(color);
        }
        if let Some(plate_model) = archive.stl_model.take() {
            if let Some(model) = &mut model {
                model.objects.extend(plate_model.objects);
            } else {
                model = Some(plate_model);
            }
        }
        archives.push((plate.id, archive));
    }
    let model = model.ok_or_else(|| invalid("sliced export requires captured STL model"))?;
    let mut identify_ids = vec![None; model.objects.len()];
    for (_, archive) in &archives {
        for object in &archive.objects {
            let model_index = model
                .objects
                .iter()
                .position(|candidate| candidate.name == object.name)
                .ok_or_else(|| invalid("captured plate does not match model objects"))?;
            let id = identify_ids
                .get_mut(model_index)
                .ok_or_else(|| invalid("captured plate does not match model objects"))?;
            if id.replace(object.identify_id).is_some() {
                return Err(invalid("captured plate does not match model objects"));
            }
        }
    }
    if identify_ids.iter().any(Option::is_none) {
        return Err(invalid("captured plate does not match model objects"));
    }
    // PartPlate.cpp:8422 allocates PrintObject IDs after the final arranged
    // ModelInstance; the copied STL object contributes object/config/volume/
    // instance/profile IDs before the first PrintObject.
    let first_print_object_id = identify_ids
        .iter()
        .flatten()
        .max()
        .copied()
        .expect("arranged archive has model instances")
        + 19;
    let sliced = archives
        .iter()
        .zip(&plates)
        .map(|((plate, archive), source)| SlicedMembers {
            plate: *plate,
            object_ids: archive
                .objects
                .iter()
                .map(|object| {
                    let model_index = model
                        .objects
                        .iter()
                        .position(|candidate| candidate.name == object.name)
                        .expect("captured plate object has a model geometry");
                    (model_index, object.identify_id)
                })
                .collect(),
            plate_json: xml::plate_json(archive, first_print_object_id + source.id - 1),
            slice_info: xml::slice_info(archive, &source.processed, *plate).into_bytes(),
            gcode: source.processed.gcode.clone(),
            previews: None,
        })
        .collect();
    write_package(
        &model.objects,
        archives[0].1.project_settings.clone(),
        Some(Sliced {
            members: sliced,
            skip_model: false,
            auxiliary: Vec::new(),
        }),
        (model.orca_version.as_deref(), None),
        None,
    )
}

fn write_package(
    objects: &[geometry::Object],
    settings: Vec<u8>,
    sliced: Option<Sliced>,
    (orca_version, saved_instance_id): (Option<&str>, Option<u32>),
    scene: Option<&scene::SceneLayout<'_>>,
) -> Result<Vec<u8>, SliceError> {
    let mut next_id = 1;
    let objects = objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            let id = next_id + object.parts.len() as u32;
            next_id = id + 1;
            let backup_id = index as u32 + 1;
            ObjectEntry {
                object,
                id,
                backup_id,
                path: format!("3D/Objects/{}_{backup_id}.model", object.name),
            }
        })
        .collect::<Vec<_>>();
    let skip_model = sliced.as_ref().is_some_and(|sliced| sliced.skip_model);
    let (mut sliced, auxiliary) = sliced.map_or((None, Vec::new()), |sliced| {
        (Some(sliced.members), sliced.auxiliary)
    });
    let sliced_output = sliced.is_some();
    let model_settings = xml::model_settings(
        &objects,
        sliced.as_deref(),
        saved_instance_id,
        scene,
        skip_model,
    )
    .into_bytes();
    let has_previews = sliced
        .as_ref()
        .is_some_and(|members| members.iter().any(|member| member.previews.is_some()));
    let mut entries = vec![(
        "[Content_Types].xml".to_owned(),
        xml::content_types().into_bytes(),
    )];
    if let Some(sliced) = &mut sliced {
        for member in sliced {
            if let Some(PlatePreviews {
                primary,
                small,
                secondary,
            }) = member.previews.take()
            {
                entries.push((PREVIEW_REFERENCES[0].1.to_owned(), primary));
                entries.push((SMALL_PREVIEW.to_owned(), small));
                entries.extend(
                    PREVIEW_REFERENCES[1..]
                        .iter()
                        .zip(secondary)
                        .map(|((_, path), bytes)| (path.to_string(), bytes)),
                );
            }
            entries.push((
                format!("Metadata/plate_{}.json", member.plate),
                member.plate_json.clone(),
            ));
        }
    }
    // bbs_3mf.cpp:7404,7437,7465: SkipModel writes no object resources, build items, object
    // files or their relationships.
    let model_objects = if skip_model { &[][..] } else { &objects[..] };
    entries.push((
        MODEL.to_owned(),
        xml::main_model(model_objects, orca_version).into_bytes(),
    ));
    if !skip_model {
        entries.push((
            "3D/_rels/3dmodel.model.rels".to_owned(),
            xml::relationships(objects.iter().map(|object| object.path.as_str())).into_bytes(),
        ));
    }
    entries.extend(
        model_objects
            .iter()
            .map(|object| (object.path.clone(), xml::mesh_model(object).into_bytes())),
    );
    entries.push((SETTINGS.to_owned(), settings));
    let (slice_info, filament_sequence) = if let Some(sliced) = sliced {
        let slice_info = xml::slice_info_many(&sliced).into_bytes();
        let filament_sequence = xml::filament_sequence(sliced.len());
        // bbs_3mf.cpp:6789-6822 writes all checksums before any G-code.
        for member in &sliced {
            entries.push((
                format!("Metadata/plate_{}.gcode.md5", member.plate),
                format!("{:X}", Md5::digest(&member.gcode)).into_bytes(),
            ));
        }
        // bbs_3mf.cpp:9005-9026 stages G-code in parallel. Keep plate-index
        // order in Ares so the archive doesn't depend on scheduling.
        let mut paths = Vec::with_capacity(sliced.len());
        for member in sliced {
            let path = format!("Metadata/plate_{}.gcode", member.plate);
            paths.push(path.clone());
            entries.push((path, member.gcode));
        }
        entries.push((
            "Metadata/_rels/model_settings.config.rels".to_owned(),
            xml::gcode_relationship(paths.iter().map(String::as_str)).into_bytes(),
        ));
        (slice_info, filament_sequence)
    } else if let Some(scene) = scene {
        (
            xml::slice_info_header().into_bytes(),
            scene.filament_sequence(),
        )
    } else {
        (
            xml::slice_info_header().into_bytes(),
            b"{\"plate_1\":{\"nozzle_sequence\":[],\"optimal_assignment\":[],\"sequence\":[]}}"
                .to_vec(),
        )
    };
    entries.extend([
        ("Metadata/model_settings.config".to_owned(), model_settings),
        ("Metadata/slice_info.config".to_owned(), slice_info),
        (
            "Metadata/filament_sequence.json".to_owned(),
            filament_sequence,
        ),
    ]);
    entries.extend(auxiliary);
    entries.push((
        "_rels/.rels".to_owned(),
        xml::root_relationships(has_previews).into_bytes(),
    ));
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
    entries: Vec<(String, Vec<u8>)>,
) -> Result<W, SliceError> {
    // Native miniz supplies local wall time when absent; the ZIP epoch keeps
    // the envelope repeatable without changing a member payload.
    let options = SimpleFileOptions::default().last_modified_time(zip::DateTime::default());
    for (name, bytes) in entries {
        // bbs_3mf.cpp:6912–6919, 6959–7024: PNGs are stored, including
        // generated plate previews. File-backed WebP pictures use Deflate.
        let method = if name.ends_with(".png") {
            zip::CompressionMethod::Stored
        } else {
            zip::CompressionMethod::Deflated
        };
        writer
            .start_file(name, options.compression_method(method))
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
