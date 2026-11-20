use super::{ArrangeOrigin, ArrangeScope, ArrangeSettings, PlaterSession};
use crate::{Project, SliceError};

pub(crate) fn arrange_cli_stl(project: Project) -> Result<Project, SliceError> {
    let mut session = PlaterSession::with_project(Vec::new(), project);
    let settings = ArrangeSettings {
        avoid_extrusion_cali_region: false,
        ..session.default_arrange_settings()
    };
    session.arrange_job(ArrangeScope::All, settings, ArrangeOrigin::Cli)?;
    let edits = session.edits();
    session.project.apply_plater_edits(&edits);
    Ok(session.project)
}
