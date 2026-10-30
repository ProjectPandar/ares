//! Selected-profile project loading and placement
//! (`OrcaSlicer.cpp:1624-1640,4376-4392`).

use super::{
    ProjectSource,
    source::ProfileInput,
    state::{self, ProjectSliceState},
};
use crate::{Project, SliceError, profiles::Placement, project::SelectedProfile};

pub(crate) fn validate_profile_source(
    source: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<(), SliceError> {
    let project = load_with_profile(source, profile)?;
    project
        .settings()
        .validate_loaded_config(&project.documents().project_settings_raw, true)?;
    crate::print::validate::validate_loaded_project(
        project,
        source.plate,
        Some(
            &profile
                .composed
                .settings()
                .project
                .preset
                .printer_settings_id
                .0,
        ),
    )
}

/// Loads the model with the selected profiles applied: `.3mf` inputs load
/// model and embedded config; other inputs stay on the STL import path.
pub(super) fn load_with_profile(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<Project, SliceError> {
    load_with_profile_preparation(project, profile).map(|(loaded, _)| loaded)
}

fn load_with_profile_preparation(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
    let selected = SelectedProfile {
        settings: profile.composed.settings(),
        profile_raw: profile.raw,
        filament_raws: profile.filament_raws,
        validate_print: project.validate_print,
    };
    match crate::InputFormat::detect(project.bytes) {
        crate::InputFormat::ThreeMf => Project::from_3mf_with_profiles(
            project.bytes,
            selected,
            project.overrides,
            project.plate,
            project.edits,
        ),
        _ => Project::from_stl_with_profiles(
            project.bytes,
            profile.filename,
            selected,
            project.overrides,
        ),
    }
}

/// Loads with profiles and applies placement without slicing — the exact
/// load+placement prefix of `prepare_profile_slice`.
pub(crate) fn load_and_place(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<Project, SliceError> {
    load_and_place_with_preparation(project, profile).map(|(loaded, _)| loaded)
}

fn load_and_place_with_preparation(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
    let (mut loaded, preparation) = load_with_profile_preparation(project, profile)?;
    if profile.placement == Placement::Center {
        let bed = bed_center(&profile.composed.settings().printer.remaining.printable_area);
        loaded.center_instances_around_point(bed);
        loaded.drop_instances_to_ground();
    }
    Ok((loaded, preparation))
}

/// Prepares the slice state for a selected-profile request: the loaded
/// project enters the ordinary plate-selection lifecycle, with the
/// requested placement applied first.
pub(super) fn prepare_profile_slice(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<ProjectSliceState, SliceError> {
    let (loaded, preparation) = load_and_place_with_preparation(project, profile)?;
    // `OrcaSlicer.cpp:2575-2620,:2653-2654`: the compatible-printers
    // gate fires after load and before parameter validation —
    // loaded-names adjudication for the `--load-settings` composite
    // (branches 1-2); the catalog/profile entry has no CLI file loop
    // (its adjudication semantics are a deferred slice — no gate).
    if let Some(names) = profile.load_names {
        super::compatibility::reject_incompatible_load_settings(
            &loaded.documents().project_settings_raw,
            names,
        )?;
    }
    loaded.settings().validate_loaded_config(
        &loaded.documents().project_settings_raw,
        project.validate_print,
    )?;
    state::prepare_loaded_project_slice_with_validation(
        loaded,
        project.plate,
        Some(
            &profile
                .composed
                .settings()
                .project
                .preset
                .printer_settings_id
                .0,
        ),
        project.validate_print,
        Some((preparation, project.overrides)),
    )
}

/// `BuildVolume::bed_center()` (`BuildVolume.hpp:83`): the printable-area
/// polygon's bounding-box center.
fn bed_center(printable_area: &crate::Point2dList) -> [f64; 2] {
    let Some(first) = printable_area.0.first() else {
        return [0.0, 0.0];
    };
    let (min_x, max_x, min_y, max_y) = printable_area.0.iter().skip(1).fold(
        (first.x, first.x, first.y, first.y),
        |(min_x, max_x, min_y, max_y), point| {
            (
                min_x.min(point.x),
                max_x.max(point.x),
                min_y.min(point.y),
                max_y.max(point.y),
            )
        },
    );
    [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5]
}
