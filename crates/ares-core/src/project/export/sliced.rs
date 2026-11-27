//! Single-plate sliced 3MF export: the CLI `--slice 1 --export-3mf` package (`OrcaSlicer.cpp:8383-8403`)
//! of a direct STL or a saved project, optionally as the `--min-save` print-job file
//! `Plater::send_gcode` also writes (`Plater.cpp:21240`).

use super::super::{ArchiveLimits, LoadedProfileProject, ProjectArchive};
use super::{SavedModel, Sliced, SlicedMembers, deferred, geometry, invalid, write_package, xml};
use crate::{
    ConfigOverrides, GenerationMetadata, ProcessedGCode, Project, SliceError, SlicerContext,
};

pub fn export_sliced_3mf(
    input: &[u8],
    filename: &str,
    mut processed: ProcessedGCode,
    first_print_object_id: u32,
    min_save: bool,
) -> Result<Vec<u8>, SliceError> {
    if filename.is_empty() || filename.contains(['/', '\\', '\0']) {
        return Err(invalid(
            "export requires a source filename without a directory",
        ));
    }
    let model = if let Some(archive) = processed.archive.as_mut()
        && let Some(model) = archive.stl_model.take()
    {
        if archive.filament_color.is_empty()
            && let Some(color) = processed.result.extruder_colors.first()
        {
            archive.filament_color.clone_from(color);
        }
        model
    } else if processed.archive.is_none() {
        return Err(invalid("sliced export requires captured plate data"));
    } else {
        if !filename.to_ascii_lowercase().ends_with(".3mf") {
            return Err(deferred(
                "sliced export requires a saved 3MF or a captured STL model",
            ));
        }
        geometry::load_saved(input)?
    };
    write_sliced_3mf_package(model, processed, first_print_object_id, min_save)
}

fn write_sliced_3mf_package(
    model: SavedModel,
    mut processed: ProcessedGCode,
    first_print_object_id: u32,
    min_save: bool,
) -> Result<Vec<u8>, SliceError> {
    let archive = processed
        .archive
        .take()
        .ok_or_else(|| invalid("sliced export requires captured plate data"))?;
    let mut identify_ids = vec![None; model.objects.len()];
    let mismatch = || invalid("captured plate does not match model objects");
    for object in &archive.objects {
        let id = identify_ids
            .get_mut(object.model_index)
            .ok_or_else(mismatch)?;
        if id.replace(object.identify_id).is_some() {
            return Err(mismatch());
        }
    }
    if identify_ids.iter().any(Option::is_none) {
        return Err(mismatch());
    }
    let sliced = SlicedMembers {
        plate: 1,
        object_ids: archive
            .objects
            .iter()
            .map(|object| (object.model_index, object.identify_id))
            .collect(),
        plate_json: xml::plate_json(&archive, first_print_object_id),
        slice_info: xml::slice_info(&archive, &processed, 1).into_bytes(),
        gcode: processed.gcode,
    };
    write_package(
        &model.objects,
        archive.project_settings,
        Some(Sliced {
            members: vec![sliced],
            skip_model: min_save,
        }),
        (model.orca_version.as_deref(), None),
        None,
    )
}

/// Single-open saved 3MF slice + export: opens the source archive once to validate
/// the saved object layout, extract geometry for export, and slice plate 1; `min_save`
/// adds `SaveStrategy::SkipModel`.
pub async fn export_saved_sliced_3mf(
    input: &[u8],
    filename: &str,
    min_save: bool,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    export_saved_sliced(input, filename, min_save, false, context)
}

/// Direct CLI sliced export checks the saved producer version unless explicitly allowed.
pub fn export_saved_sliced_3mf_for_cli(
    input: &[u8],
    filename: &str,
    min_save: bool,
    allow_newer_file: bool,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    export_saved_sliced(input, filename, min_save, !allow_newer_file, context)
}

fn export_saved_sliced(
    input: &[u8],
    filename: &str,
    min_save: bool,
    check_version: bool,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    if input.is_empty() {
        return Err(SliceError::EmptyInput);
    }
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    let loaded = LoadedProfileProject::open_from_archive(&mut archive, check_version)?;
    let first_id = saved_print_object_id(loaded.project())
        .map_err(|detail| SliceError::InvalidInput(detail.into()))?;
    let saved_model = geometry::from_saved_project(archive, loaded.project());
    let processed = crate::project_slice::slice_loaded_project_plate_with_archive(
        (input, Box::new(loaded)),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        context,
    )?;
    if filename.is_empty() || filename.contains(['/', '\\', '\0']) {
        return Err(invalid(
            "export requires a source filename without a directory",
        ));
    }
    if !filename.to_ascii_lowercase().ends_with(".3mf") {
        return Err(deferred(
            "sliced export requires a saved 3MF or a captured STL model",
        ));
    }
    write_sliced_3mf_package(saved_model?, processed, first_id, min_save)
}

fn saved_print_object_id(project: &Project) -> Result<u32, &'static str> {
    if project.plates().len() != 1 {
        return Err("sliced export requires one saved plate");
    }
    let objects = project.objects();
    let mut volumes = 0;
    for object in objects {
        let [instance] = object.instances() else {
            return Err("sliced export requires one saved instance per object");
        };
        if !instance.printable()
            || object
                .volumes()
                .iter()
                .any(|volume| volume.volume_type() != crate::ProjectVolumeType::ModelPart)
        {
            return Err("sliced export requires printable instances of ordinary model parts");
        }
        volumes += object.volumes().len() as u32;
    }

    Ok(print_object_id(objects.len() as u32, volumes, true))
}

/// The CLI's first `PrintObject` ID after loading `objects` model objects with `volumes` volumes.
pub fn print_object_id(objects: u32, volumes: u32, saved: bool) -> u32 {
    // The direct STL path uses one fresh model object and one volume. The saved
    // path starts with the same ObjectBase allocation sequence.
    let model = 2; // Model + ModelWipeTower (Model.hpp:1704,1753).
    let config = 1; // ModelConfigObject (Model.hpp:86–104).
    let object = 1 + config + 1; // ModelObject + config + LayerHeightProfile (582–588).
    let instance = 1; // ModelInstance (1562–1567).
    let volume = 1 + config + 4 + 8; // ModelVolume, config and FacetsAnnotations (904–935).
    let input = 2 * model + (object + instance) * objects + volume * volumes;
    // The reader's return and the CLI vector insertion each construct a tower
    // but preserve the existing model ID (Model.cpp:489–516; OrcaSlicer.cpp:2008).
    let model_moves = 2;
    let print = 1 + model; // PrintBase and its private model (PrintBase.hpp:390,561).
    let initial_list = 3 + print; // PartPlateList and two PartPlates (PartPlate.cpp:5966–5995).
    let plate_preparation = if saved {
        1 + print // Replacement plate + Print (PartPlate.cpp:6589–6605).
    } else {
        2 // Two more model allocations are avoided when no saved plate is restored.
    };
    let slice_copy = model; // Default construct before copying IDs (OrcaSlicer.cpp:6466–6470).
    let print_object = 1; // Print::apply (PrintApply.cpp:1977).
    input + model_moves + initial_list + plate_preparation + slice_copy + print_object
}
