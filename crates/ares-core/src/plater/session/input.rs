//! Original slicing input, separate from the edited model.

use super::PlaterSession;
use crate::profiles::entry::ProfileSettings;
use crate::{ProfileSliceRequest, Project, SliceError};

pub(in crate::plater) enum SessionInput {
    Bytes(Vec<u8>),
    Model(Box<Project>),
}

impl PlaterSession {
    pub fn input(&self) -> Option<&[u8]> {
        match &self.input {
            SessionInput::Bytes(bytes) => Some(bytes),
            SessionInput::Model(_) => None,
        }
    }

    pub(in crate::plater) fn source(&self) -> crate::project_slice::ProjectInput<'_> {
        match &self.input {
            SessionInput::Bytes(bytes) => crate::project_slice::ProjectInput::Bytes(bytes),
            SessionInput::Model(model) => crate::project_slice::ProjectInput::Model(model),
        }
    }

    pub async fn profile_settings(
        &self,
        request: ProfileSliceRequest<'_>,
    ) -> Result<ProfileSettings, SliceError> {
        crate::profiles::with_edited_profile_source(
            self.source(),
            request,
            &super::super::NO_EDITS,
            |source| {
                let project = crate::project_slice::load_profile_project(&source)?;
                Ok(ProfileSettings(
                    project.documents().project_settings_raw.clone(),
                ))
            },
        )
    }
}
