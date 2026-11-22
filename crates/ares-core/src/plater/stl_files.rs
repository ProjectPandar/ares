//! Plater::priv::load_files multipart choices (Plater.cpp:10075–10125).

use super::{PlaterSession, session::SessionInput};
use crate::{ProfileRequest, Project, SliceError};

#[derive(Clone, Copy, Debug)]
pub struct StlFile<'a> {
    pub name: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct StlImportOptions {
    pub as_multipart: bool,
    pub auto_drop: bool,
}

impl PlaterSession {
    pub fn load_stl_files(
        files: &[StlFile<'_>],
        options: StlImportOptions,
    ) -> Result<Self, SliceError> {
        let mut project = Project::from_stl_files(files)?;
        project.place_stl_files(options);
        Ok(Self::with_stl_model(project))
    }

    pub async fn load_stl_files_with_profiles<'a>(
        files: &[StlFile<'_>],
        options: StlImportOptions,
        request: impl ProfileRequest<'a>,
    ) -> Result<Self, SliceError> {
        let model = Project::from_stl_files(files)?;
        let mut project = crate::profiles::with_edited_profile_source(
            crate::project_slice::ProjectInput::Model(&model),
            (request.request(), request.composition()),
            &super::NO_EDITS,
            |source| crate::project_slice::load_profile_project(&source),
        )?;
        project.place_stl_files(options);
        Ok(Self::with_stl_model(project))
    }

    fn with_stl_model(project: Project) -> Self {
        let original = project.copy_stl_model();
        Self::from_source(SessionInput::Model(Box::new(original)), project)
    }
}
