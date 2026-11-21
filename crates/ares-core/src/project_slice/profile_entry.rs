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
    load_with_profile_preparation(project, profile, None).map(|(loaded, _)| loaded)
}

fn load_with_profile_preparation(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
    upward_report: Option<&mut Vec<String>>,
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
    let selected = SelectedProfile {
        settings: profile.composed.settings(),
        profile_raw: profile.raw,
        filament_raws: profile.filament_raws,
        validate_print: project.validate_print,
        load_names: profile.load_names,
        retained_refresh: profile.retained_refresh,
        full_fff_config: profile.full_fff_config,
    };
    let (mut loaded, preparation) = match project.input {
        super::ProjectInput::Assembly(assembly) => match &assembly.0 {
            crate::project::assembly::AssemblySource::Archive(file) => {
                let (mut loaded, preparation) = Project::from_3mf_with_profiles(
                    file.bytes,
                    selected,
                    project.overrides,
                    (None, project.edits),
                    upward_report,
                )?;
                loaded.assemble_loaded_archive()?;
                Ok((loaded, preparation))
            }
            crate::project::assembly::AssemblySource::Model(model) => {
                let (mut loaded, preparation) =
                    model.with_stl_profiles(selected, project.overrides)?;
                loaded.place_assembly_plates();
                Ok((loaded, preparation))
            }
        },
        super::ProjectInput::OptionStl { .. } | super::ProjectInput::ArrangedStl(..) => {
            unreachable!("not a selected-profile input")
        }
        super::ProjectInput::Model(model) => model
            .with_stl_profiles(selected, project.overrides)
            .map(|(mut loaded, preparation)| {
                loaded.apply_plater_edits(project.edits);
                (loaded, preparation)
            }),
        super::ProjectInput::ImportedStl(imported) => {
            Project::from_imported_stl_with_profiles(imported, selected, project.overrides).map(
                |(mut loaded, preparation)| {
                    loaded.apply_plater_edits(project.edits);
                    (loaded, preparation)
                },
            )
        }
        super::ProjectInput::Bytes(bytes)
            if crate::InputFormat::detect(bytes) == crate::InputFormat::ThreeMf =>
        {
            Project::from_3mf_with_profiles(
                bytes,
                selected,
                project.overrides,
                (project.plate, project.edits),
                upward_report,
            )
        }
        super::ProjectInput::Bytes(bytes) => {
            Project::from_stl_with_profiles(bytes, profile.filename, selected, project.overrides)
                .map(|(mut loaded, preparation)| {
                    loaded.apply_plater_edits(project.edits);
                    (loaded, preparation)
                })
        }
    }?;
    if profile.full_fff_config.is_some() {
        loaded.compose_as_full_fff_config();
    }
    Ok((loaded, preparation))
}

/// Loads with profiles and applies placement without slicing — the exact
/// load+placement prefix of `prepare_profile_slice`.
pub(crate) fn load_and_place(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<Project, SliceError> {
    load_and_place_with_preparation(project, profile, None).map(|(loaded, _)| loaded)
}

fn load_and_place_with_preparation(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
    upward_report: Option<&mut Vec<String>>,
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
    let (mut loaded, preparation) = load_with_profile_preparation(project, profile, upward_report)?;
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
    upward_report: Option<&mut Vec<String>>,
) -> Result<ProjectSliceState, SliceError> {
    let (loaded, preparation) = load_profile_slice(project, profile, upward_report)?;
    state::prepare_loaded_project_slice_with_validation(
        loaded,
        project.plate,
        Some(profile.printer_name()),
        (project.validate_print, project.max_triangle_count),
        Some((&preparation, project.overrides)),
    )
}

pub(crate) fn load_profile_slice(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
    upward_report: Option<&mut Vec<String>>,
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
    let (mut loaded, preparation) =
        load_and_place_with_preparation(project, profile, upward_report)?;
    if let Some(custom_gcodes) = project.custom_gcodes.clone() {
        loaded.replace_plate_custom_gcodes(custom_gcodes);
    }
    // `OrcaSlicer.cpp:2575-2620,:2653-2654`: the compatible-printers
    // gate fires after load and before parameter validation —
    // loaded-names adjudication for the `--load-settings` composite
    // (branches 1-2); the catalog/profile entry has no CLI file loop
    // (its adjudication semantics are a deferred slice — no gate).
    if let Some(names) = profile.load_names
        && names.embedded.is_none()
    {
        super::compatibility::reject_incompatible_load_settings(
            &loaded.documents().project_settings_raw,
            names,
            profile.raw,
        )?;
    }
    if project.validate_config {
        loaded.settings().validate_loaded_config(
            &loaded.documents().project_settings_raw,
            project.validate_print,
        )?;
    }
    if project.orient {
        crate::plater::apply_cli_orientation(&mut loaded, project.plate)?;
    }
    if profile.placement == Placement::Arrange
        && matches!(project.input, super::ProjectInput::Model(_))
    {
        loaded = crate::plater::arrange_cli_stl(loaded)?;
    }
    Ok((loaded, preparation))
}

/// `BuildVolume::bed_center()` (`BuildVolume.hpp:83`): the printable-area
/// polygon's bounding-box center.
pub(crate) fn center_profile_project(project: &mut Project) {
    let bed = bed_center(&project.settings().printer.remaining.printable_area);
    project.center_instances_around_point(bed);
    project.drop_instances_to_ground();
}

pub(super) fn bed_center(printable_area: &crate::Point2dList) -> [f64; 2] {
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
