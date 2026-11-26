use super::{deferred, geometry, project_settings, write_package};
use crate::{LoadSettingsRequest, SliceError};

pub(super) fn export_saved(
    archive: super::super::ProjectArchive<'_>,
    input: &[u8],
    presets: LoadSettingsRequest<'_>,
) -> Result<Vec<u8>, SliceError> {
    let project = crate::profiles::project_for_3mf_export(input, &presets)?;
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
