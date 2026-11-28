use super::{deferred, geometry, invalid, project_settings, write_package};
use crate::{
    ConfigSubstitution, LoadSettingsRequest, SliceError,
    project::{ArchiveLimits, LoadedProfileProject, ProjectArchive},
};

/// Unsliced export input, with the first positional 3MF loaded before preset selection.
pub struct CliUnsliced3mf<'a> {
    input: &'a [u8],
    filename: &'a str,
    saved: Option<(ProjectArchive<'a>, Box<LoadedProfileProject>)>,
}

impl<'a> CliUnsliced3mf<'a> {
    pub fn admit(input: &'a [u8], filename: &'a str) -> Result<Self, SliceError> {
        if filename.is_empty() || filename.contains(['/', '\\', '\0']) {
            return Err(invalid(
                "export requires a source filename without a directory",
            ));
        }
        let mut saved = None;
        if filename.to_ascii_lowercase().ends_with(".3mf") {
            let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
            if archive
                .paths()
                .any(|path| path.as_str() == "Metadata/model_settings.config")
            {
                let loaded = LoadedProfileProject::open_for_export(&mut archive)?;
                loaded.validate_post_process()?;
                saved = Some((archive, Box::new(loaded)));
            }
        }
        Ok(Self {
            input,
            filename,
            saved,
        })
    }

    pub fn substitutions(&self) -> &[ConfigSubstitution] {
        self.saved
            .as_ref()
            .map_or(&[], |(_, loaded)| loaded.substitutions())
    }

    pub fn export(self, presets: LoadSettingsRequest<'_>) -> Result<Vec<u8>, SliceError> {
        if let Some((archive, loaded)) = self.saved {
            return export_saved(archive, self.input, loaded, presets);
        }
        let object =
            geometry::Object::unsliced(geometry::load(self.input, self.filename)?, self.filename);
        let settings = project_settings::write(&presets)?;
        write_package(&[object], settings, None, (None, None), None)
    }
}

fn export_saved(
    archive: ProjectArchive<'_>,
    input: &[u8],
    loaded: Box<LoadedProfileProject>,
    presets: LoadSettingsRequest<'_>,
) -> Result<Vec<u8>, SliceError> {
    let project = crate::profiles::project_for_3mf_export(input, &presets, loaded)?;
    let [object] = project.objects() else {
        return Err(deferred("expected one saved object"));
    };
    let [instance] = object.instances() else {
        return Err(deferred("expected one saved instance"));
    };
    if project.plates().len() != 1 || object.volumes().len() != 1 {
        return Err(deferred("expected one saved plate and ordinary part"));
    }
    let saved = geometry::from_saved_project(archive, &project)?;
    let raw = &project.documents().project_settings_raw;
    project_settings::require_single_nozzle(raw)?;
    let settings = project_settings::write_raw(raw, false)?;
    // Without a loaded ID the writer uses the instance's ObjectBase ID
    // (bbs_3mf.cpp:8596–8601). Two Models allocate two IDs each; the object
    // allocates three before the instance (Model.hpp:582–588,1562–1567,1704–1753).
    let id = instance
        .saved_identify_id()
        .filter(|&id| id > 0)
        .unwrap_or(8);
    write_package(
        &saved.objects,
        settings,
        None,
        (saved.orca_version.as_deref(), Some(id)),
        None,
    )
}
