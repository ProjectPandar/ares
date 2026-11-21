use super::{Project, ProjectSettingsRaw, SliceError, load_project_raw, settings};

pub(crate) struct LoadedProfileProject {
    project: Project,
    sparse_settings: ProjectSettingsRaw,
    preparation: crate::options::flush_matrix::PreparationContext,
}

impl LoadedProfileProject {
    pub(crate) fn load(input: &[u8]) -> Result<Self, SliceError> {
        let mut sparse_settings = ProjectSettingsRaw::default();
        let (project, preparation) = load_project_raw(
            input,
            &crate::ConfigOverrides::empty(),
            true,
            Some(&mut sparse_settings),
        )?;
        crate::project_slice::validate_post_process(&project)?;
        Ok(Self {
            project,
            sparse_settings,
            preparation,
        })
    }

    pub(crate) fn into_profile_project(
        self,
    ) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
        let Self {
            mut project,
            sparse_settings,
            preparation,
        } = self;
        let (raw, settings) =
            settings::resolve(sparse_settings, &crate::ConfigOverrides::empty(), false)?;
        project.set_embedded_settings(raw, settings);
        Ok((project, preparation))
    }
}
