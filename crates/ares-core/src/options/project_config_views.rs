mod retract;

use super::{GCodeOptions, ProjectSettings};
use crate::SliceError;

#[derive(Debug, PartialEq)]
pub(crate) struct ProjectConfigViews {
    pub(crate) full: ProjectSettings,
    pub(crate) runtime: ProjectSettings,
    pub(crate) runtime_gcode: GCodeOptions,
}

pub(crate) fn resolve_project_config_views(
    full: ProjectSettings,
) -> Result<ProjectConfigViews, SliceError> {
    let mut runtime = full.clone();
    retract::apply(&mut runtime, &full)?;
    let runtime_gcode = GCodeOptions::from_sources(
        &runtime.printer.gcode,
        &runtime.process.gcode,
        &runtime.filament.gcode,
        &runtime.project.gcode,
    );
    Ok(ProjectConfigViews {
        full,
        runtime,
        runtime_gcode,
    })
}
