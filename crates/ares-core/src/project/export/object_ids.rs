//! `ObjectBase` ID arithmetic (`Model.hpp`, `PrintBase.hpp`, `PartPlate.cpp`,
//! `PrintApply.cpp`, `OrcaSlicer.cpp`) and single-open saved-project sliced 3MF export.
use crate::{
    ConfigOverrides, GenerationMetadata, Project, ProjectVolumeType, SliceError, SlicerContext,
};

use super::super::{ArchiveLimits, LoadedProfileProject, ProjectArchive};
use super::{geometry, invalid, write_sliced_3mf_package};

/// First `PrintObject` `ObjectBase` ID allocated on the direct single-STL export path.
pub fn direct_stl_print_object_id() -> u32 {
    print_object_id(1, 1, false)
}

/// Validates a saved 3MF project for sliced export and computes its first
/// `PrintObject` `ObjectBase` ID.
pub fn saved_print_object_id(project: &Project) -> Result<u32, SliceError> {
    if project.plates().len() != 1 {
        return Err(invalid("sliced export requires one saved plate"));
    }
    let objects = project.objects();
    let mut volumes = 0;
    for object in objects {
        let [instance] = object.instances() else {
            return Err(invalid(
                "sliced export requires one saved instance per object",
            ));
        };
        if !instance.printable()
            || object
                .volumes()
                .iter()
                .any(|volume| volume.volume_type() != ProjectVolumeType::ModelPart)
        {
            return Err(invalid(
                "sliced export requires printable instances of ordinary model parts",
            ));
        }
        volumes += object.volumes().len() as u32;
    }

    Ok(print_object_id(objects.len() as u32, volumes, true))
}

fn print_object_id(objects: u32, volumes: u32, saved: bool) -> u32 {
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

/// Single-open saved 3MF slice + export: opens the source archive once to validate
/// the saved object layout, extract geometry for export, and slice plate 1.
pub fn export_saved_sliced_3mf(
    input: &[u8],
    filename: &str,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    if input.is_empty() {
        return Err(SliceError::EmptyInput);
    }
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    let loaded = LoadedProfileProject::open_from_archive(&mut archive, false)?;
    let first_id = saved_print_object_id(loaded.project())?;
    let saved_model = geometry::from_saved_project(archive, loaded.project());
    let processed = crate::project_slice::slice_loaded_project_plate_with_archive(
        (input, Box::new(loaded)),
        metadata,
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
        return Err(super::deferred(
            "sliced export requires a saved 3MF or a captured STL model",
        ));
    }
    write_sliced_3mf_package(saved_model?, processed, first_id)
}
