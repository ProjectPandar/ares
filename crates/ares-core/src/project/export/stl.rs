use super::{
    direct_stl_settings, invalid, placed_stl_model, write_package, write_project_settings_raw,
};
use crate::profiles::entry::load_settings::with_load_settings_source;
use crate::project_slice::{ProjectInput, load_and_place};
use crate::{ImportedStl, LoadSettingsRequest, LoadSettingsSliceRequest, Placement, SliceError};

/// Export a single imported STL with the same preset composition used by direct slicing.
pub fn export_imported_stl_3mf(
    model: &ImportedStl,
    presets: LoadSettingsRequest<'_>,
    placement: Placement,
) -> Result<Vec<u8>, SliceError> {
    let filename = model.volume().name();
    if filename.is_empty() || filename.contains(['/', '\\', '\0']) {
        return Err(invalid(
            "export requires a source filename without a directory",
        ));
    }
    let request = LoadSettingsSliceRequest {
        filename,
        catalog_json: None,
        loaded_sources: presets.loaded_sources,
        uptodate: false,
        settings: presets.settings,
        filaments: presets.filaments,
        plate: None,
        overrides: &Default::default(),
        placement,
        orient: false,
        sliced_plate: &mut Default::default(),
    };
    with_load_settings_source(
        (ProjectInput::ImportedStl(model), &[], None),
        request,
        None,
        |source| {
            let project =
                load_and_place(&source, source.profile.expect("loaded settings profile"))?;
            let model = placed_stl_model(&project)?;
            let settings = write_project_settings_raw(&direct_stl_settings(&project), false)?;
            let identify_id = (placement != Placement::None)
                .then(|| project.objects()[0].instances()[0].labeled_id());
            write_package(&model.objects, settings, None, (None, identify_id), None)
        },
    )
}
