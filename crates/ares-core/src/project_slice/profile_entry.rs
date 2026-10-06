//! Selected-profile project loading and placement
//! (`OrcaSlicer.cpp:1624-1640,4376-4392`).

use super::{
    ProjectSource,
    source::ProfileInput,
    state::{self, ProjectSliceState},
};
use crate::{Project, SliceError, profiles::Placement, project::SelectedProfile};

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
/// project enters the ordinary plate-selection lifecycle, with the
/// requested placement applied first.
pub(super) fn prepare_profile_slice(
    project: &ProjectSource<'_>,
    profile: ProfileInput<'_>,
) -> Result<ProjectSliceState, SliceError> {
    let mut loaded = load_with_profile(project, profile)?;
    // `OrcaSlicer.cpp:4376-4392` (--center): bed-center placement
    // (`BuildVolume::bed_center`, the printable-area bbox center) followed
    // by the Z ground drop.
    if profile.placement == Placement::Center {
        let bed = bed_center(&profile.composed.settings().printer.remaining.printable_area);
        loaded.center_instances_around_point(bed);
        loaded.drop_instances_to_ground();
    }
    state::prepare_loaded_project_slice(loaded, project.plate)
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
