//! Custom-gcode references whose stock input slices to completion.
use super::{CliResultReference, OrcaRunner, ReferenceRequest, ResultKind};
use crate::runner::StageError;
use std::path::Path;

impl OrcaRunner {
    pub(crate) fn custom_gcode_reference(
        &self,
        project: &Path,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(
            project,
            ReferenceRequest {
                kind: ResultKind::CustomGCode,
                load_settings: None,
            },
        )
    }

    pub(crate) fn custom_gcode_success_reference(
        &self,
        project: &Path,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(
            project,
            ReferenceRequest {
                kind: ResultKind::CustomGCodeSuccess,
                load_settings: None,
            },
        )
    }
}
