use super::{
    ExportedCase, OrcaRunner, application,
    stages::{FailureKind, Stage, StageError},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};

fn error(detail: impl ToString) -> StageError {
    StageError::new(Stage::Application, FailureKind::ApplicationMismatch, detail)
}

fn plate_mode(xml: &str) -> Result<&str, StageError> {
    let plate = xml
        .split_once("<plate>")
        .and_then(|(_, tail)| tail.split_once("</plate>"))
        .map(|(plate, _)| plate)
        .ok_or_else(|| error("missing plate"))?;
    plate
        .split_once("<metadata key=\"filament_map_mode\" value=\"")
        .and_then(|(_, tail)| tail.split_once('"'))
        .map(|(mode, _)| mode)
        .ok_or_else(|| error("missing plate filament_map_mode"))
}

pub(crate) fn verify(project: &[u8], requested: &Value) -> Result<Value, StageError> {
    let mut proof = application::verify_value(project, "filament_map_mode", requested)?;
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).map_err(error)?;
    let mut xml = String::new();
    archive
        .by_name("Metadata/model_settings.config")
        .map_err(error)?
        .read_to_string(&mut xml)
        .map_err(error)?;
    let actual = plate_mode(&xml)?;
    if requested.as_str() != Some(actual) {
        return Err(error(format!(
            "plate filament_map_mode: requested {requested}, effective {actual}"
        )));
    }
    proof["plate"] = json!({"effective": actual, "source": "bbs_3mf.cpp::_extract_model_config_from_mb_plate; plate_config overrides"});
    Ok(proof)
}

impl OrcaRunner {
    /// Activate the concrete mode at the plate owner before either slicer reads it.
    pub(crate) fn bind_filament_map_mode(
        &self,
        exported: ExportedCase,
        requested: &Value,
    ) -> Result<ExportedCase, StageError> {
        application::verify_value(&exported.project, "filament_map_mode", requested)?;
        let _: ares_core::ProjectFilamentMapMode =
            serde_json::from_value(requested.clone()).map_err(error)?;
        let requested = requested.as_str().unwrap();
        let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).map_err(error)?;
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..input.len() {
            let mut file = input.by_index(index).map_err(error)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(error)?;
            if file.name() == "Metadata/model_settings.config" {
                let xml = String::from_utf8(bytes).map_err(error)?;
                let old = plate_mode(&xml)?;
                let before = format!("<metadata key=\"filament_map_mode\" value=\"{old}\"/>");
                let after = format!("<metadata key=\"filament_map_mode\" value=\"{requested}\"/>");
                bytes = xml.replacen(&before, &after, 1).into_bytes();
            }
            output
                .start_file(
                    file.name(),
                    zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
                )
                .map_err(error)?;
            output.write_all(&bytes).map_err(error)?;
        }
        let project = output.finish().map_err(error)?.into_inner();
        verify(&project, &json!(requested))?;
        let digest: String = Sha256::digest(&project)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let slug = format!("plate-mode-{digest}");
        let project_path = self.work.join(format!("{slug}.3mf"));
        let output_dir = self.work.join(slug);
        let io_error = |error| StageError::new(Stage::Application, FailureKind::Io, error);
        std::fs::create_dir_all(&output_dir).map_err(io_error)?;
        std::fs::write(&project_path, &project).map_err(io_error)?;
        Ok(ExportedCase {
            label: exported.label,
            project,
            project_path,
            output_dir,
        })
    }
}
