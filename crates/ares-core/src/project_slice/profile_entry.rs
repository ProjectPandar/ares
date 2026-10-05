//! Selected-profile project loading (`OrcaSlicer.cpp:1624-1640`).

use super::{ProjectSource, source::ProfileInput, state};
use crate::{Project, SliceError, project::SelectedProfile};

/// Loads the model with the selected profiles applied: `.3mf` inputs load
/// model and embedded config; other inputs stay on the STL import path.
pub(super) fn load_with_profile(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<Project, SliceError> {
    let selected = SelectedProfile {
        settings: profile.composed.settings(),
        profile_raw: profile.raw,
        filament_raws: profile.filament_raws,
    };
    match crate::InputFormat::detect(project.bytes) {
        crate::InputFormat::ThreeMf => {
            Project::from_3mf_with_profiles(project.bytes, selected, project.overrides)
        }
        _ => Project::from_stl_with_profiles(
            project.bytes,
            profile.filename,
            selected,
            project.overrides,
        ),
    }
}

/// Prepares the slice state for a selected-profile request: the loaded
/// project enters the ordinary plate-selection lifecycle.
pub(super) fn prepare_profile_slice(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<state::ProjectSliceState, SliceError> {
    let loaded = load_with_profile(project, profile)?;
    state::prepare_loaded_project_slice(loaded, project.plate)
}
