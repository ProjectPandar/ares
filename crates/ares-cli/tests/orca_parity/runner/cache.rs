#[path = "cache/direct_stl.rs"]
mod direct_stl;
#[path = "cache/object_filaments.rs"]
mod object_filaments;

use object_filaments::assign_object_filaments;

use super::{
    CaseInputs, ExportedCase, OrcaRunner, ParityCase, PresetKind, apply_override, cache_identity,
    digest_slug, join_paths,
    stages::{self, FailureKind, Stage, StageError},
    write_preset,
};
use serde_json::{Map, Value, json};
use std::{fs, path::Path};

#[path = "cache/cli_result.rs"]
mod cli_result;
pub(crate) use cli_result::EarlyExit;
#[path = "cache/plate_selection.rs"]
mod plate_selection;
#[path = "cache/print_statistics.rs"]
mod print_statistics;
#[path = "cache/processor.rs"]
mod processor;

impl OrcaRunner {
    fn producer(&self) -> Result<&cache_identity::Producer, StageError> {
        if let Some(producer) = self.producer.get() {
            return Ok(producer);
        }
        let producer = cache_identity::identify(&self.bin)?;
        Ok(self.producer.get_or_init(|| producer))
    }

    pub(crate) fn export_case(
        &self,
        inputs: &CaseInputs<'_>,
        overrides: &Map<String, Value>,
        model: &Path,
    ) -> Result<ExportedCase, StageError> {
        self.export_case_multi(inputs, overrides, &[model], &[])
    }

    /// Export with several models and an explicit object→filament
    /// assignment (1-based extruder per object, in export order). The
    /// assignment is patched into the exported project's
    /// `Metadata/model_settings.config` (`<metadata key="extruder" …/>`),
    /// which both slicers read, so a test can place objects on different
    /// filaments and reach multi-tool paths (SEMM ramming, tool changes).
    pub(crate) fn export_case_multi(
        &self,
        inputs: &CaseInputs<'_>,
        overrides: &Map<String, Value>,
        models: &[&Path],
        object_filaments: &[u32],
    ) -> Result<ExportedCase, StageError> {
        let machine = apply_override(inputs.machine, overrides, PresetKind::Machine);
        let mut process = apply_override(inputs.process, overrides, PresetKind::Process);
        let compatible_empty = process
            .get("compatible_printers")
            .and_then(Value::as_array)
            .is_none_or(|printers| printers.is_empty());
        if compatible_empty && let Some(name) = machine.get("name").and_then(Value::as_str) {
            process.insert(
                "compatible_printers".to_owned(),
                Value::Array(vec![Value::String(name.to_owned())]),
            );
        }
        let filaments: Vec<_> = inputs
            .filaments
            .iter()
            .map(|filament| apply_override(filament, overrides, PresetKind::Filament))
            .collect();
        let model_bytes = stages::read(
            models.first().ok_or_else(|| {
                StageError::new(Stage::Export, FailureKind::Io, "no model inputs".to_owned())
            })?,
            Stage::Export,
        )?;
        let mut model_inputs = Vec::new();
        for model in models {
            model_inputs.push((*model, stages::read(model, Stage::Export)?));
        }
        let slug = digest_slug(
            inputs.label,
            &machine,
            &process,
            &filaments,
            (models[0], &model_bytes),
        );
        let producer = self.producer()?;
        let presets = json!({"machine":preset_value("machine", &machine),"process":preset_value("process", &process),
            "filaments":filaments.iter().map(|fields| preset_value("filament", fields)).collect::<Vec<_>>()});
        let request_components = json!({"recipe":"export-v2/arrange-1",
            "presets_sha256":cache_identity::raw_digest(&serde_json::to_vec(&presets).unwrap()),
            "model_sha256":cache_identity::raw_digest(&model_bytes),"model_name":models[0].file_name().unwrap().to_str().unwrap(),"project_name":slug,
            "extra_models":model_inputs.iter().skip(1).map(|(path, bytes)| json!({
                "name": path.file_name().unwrap().to_str().unwrap(),
                "sha256": cache_identity::raw_digest(bytes),
            })).collect::<Vec<_>>(),
            "object_filaments":object_filaments});
        let request = cache_identity::digest(&[
            b"export-v2",
            &serde_json::to_vec(&request_components).unwrap(),
        ]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("export");
        fs::create_dir_all(&parent).map_err(|error| io(Stage::Export, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), Stage::Export)?;
        let filename = format!("{slug}.3mf");
        let hit = entry.join("receipt.json").is_file() && entry.join(&filename).is_file();
        if !hit {
            let staging = tempfile::Builder::new()
                .prefix("export-")
                .tempdir_in(&parent)
                .map_err(|error| io(Stage::Export, error))?;
            let project_path = staging.path().join(&filename);
            let machine_file = staging.path().join(format!("{slug}-machine.json"));
            let process_file = staging.path().join(format!("{slug}-process.json"));
            write_preset(&machine_file, "machine", &machine)?;
            write_preset(&process_file, "process", &process)?;
            let mut filament_files = Vec::new();
            for (index, filament) in filaments.iter().enumerate() {
                let file = staging.path().join(format!("{slug}-filament-{index}.json"));
                write_preset(&file, "filament", filament)?;
                filament_files.push(file);
            }
            stages::run_live(
                &self.bin,
                [
                    "--load-settings",
                    &join_paths([&machine_file, &process_file].into_iter()),
                    "--load-filaments",
                    &join_paths(filament_files.iter()),
                    "--arrange",
                    "1",
                    "--export-3mf",
                ]
                .into_iter()
                .chain([project_path.to_str().unwrap()])
                .chain(models.iter().map(|model| model.to_str().unwrap()))
                .map(str::to_owned),
                Stage::Export,
                &self.work,
            )?;
            let project = assign_object_filaments(
                &stages::read(&project_path, Stage::Export)?,
                object_filaments,
            )?;
            fs::write(&project_path, &project).map_err(|error| io(Stage::Export, error))?;
            receipt(
                staging.path(),
                producer,
                (&request, Stage::Export, &request_components),
                &project,
                &self.work,
            )?;
            publish(staging.keep(), &entry, Stage::Export)?;
        }
        trace(
            &self.work,
            producer,
            &request,
            Stage::Export,
            if hit { "hit" } else { "miss" },
        )?;
        let project_path = entry.join(filename);
        let output_dir = entry.join(slug);
        fs::create_dir_all(&output_dir).map_err(|error| io(Stage::Export, error))?;
        fs::write(output_dir.join("input.stl"), model_bytes)
            .map_err(|error| io(Stage::Export, error))?;
        Ok(ExportedCase {
            label: inputs.label.to_owned(),
            project: stages::read(&project_path, Stage::Export)?,
            project_path,
            output_dir,
        })
    }

    /// Wraps an authored `.3mf` fixture for the slice reference stages
    /// (custom-G-code witness fixtures).
    pub(crate) fn fixture_case(
        &self,
        label: &str,
        project: Vec<u8>,
    ) -> Result<ExportedCase, StageError> {
        let directory = tempfile::Builder::new()
            .prefix("fixture-case-")
            .tempdir_in(
                crate::artifacts::root_from_env().map_err(|error| io(Stage::Export, error))?,
            )
            .map_err(|error| io(Stage::Export, error))?
            .keep();
        let project_path = directory.join("input.3mf");
        fs::write(&project_path, &project).map_err(|error| io(Stage::Export, error))?;
        let output_dir = directory.join("native");
        fs::create_dir_all(&output_dir).map_err(|error| io(Stage::Export, error))?;
        Ok(ExportedCase {
            label: label.to_owned(),
            project,
            project_path,
            output_dir,
        })
    }

    pub(crate) fn slice_case(&self, exported: &ExportedCase) -> Result<ParityCase, StageError> {
        self.cached_slice(exported, false)
    }
    pub(crate) fn rerun_reference(
        &self,
        exported: &ExportedCase,
    ) -> Result<ParityCase, StageError> {
        self.cached_slice(exported, true)
    }

    fn cached_slice(&self, exported: &ExportedCase, force: bool) -> Result<ParityCase, StageError> {
        let producer = self.producer()?;
        let project = stages::read(&exported.project_path, Stage::Slice)?;
        let request_components = json!({"recipe":"slice-v2/plate-0","project_sha256":cache_identity::raw_digest(&project),
            "project_name":exported.project_path.file_name().unwrap().to_str().unwrap()});
        let request = cache_identity::digest(&[
            b"slice-v2",
            &serde_json::to_vec(&request_components).unwrap(),
        ]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("slice");
        fs::create_dir_all(&parent).map_err(|error| io(Stage::Slice, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), Stage::Slice)?;
        let reference_path = entry.join("plate_1.gcode");
        let hit = !force && entry.join("receipt.json").is_file() && reference_path.is_file();
        if !hit {
            let staging = tempfile::Builder::new()
                .prefix("slice-")
                .tempdir_in(&parent)
                .map_err(|error| io(Stage::Slice, error))?;
            stages::run_live(
                &self.bin,
                [
                    "--slice",
                    "0",
                    "--outputdir",
                    exported.output_dir.to_str().unwrap(),
                    exported.project_path.to_str().unwrap(),
                ]
                .into_iter()
                .map(str::to_owned),
                Stage::Slice,
                &self.work,
            )?;
            let reference = stages::read(&exported.output_dir.join("plate_1.gcode"), Stage::Slice)?;
            fs::write(staging.path().join("plate_1.gcode"), &reference)
                .map_err(|error| io(Stage::Slice, error))?;
            receipt(
                staging.path(),
                producer,
                (&request, Stage::Slice, &request_components),
                &reference,
                &self.work,
            )?;
            publish(staging.keep(), &entry, Stage::Slice)?;
        }
        trace(
            &self.work,
            producer,
            &request,
            Stage::Slice,
            if force {
                "forced-rerun"
            } else if hit {
                "hit"
            } else {
                "miss"
            },
        )?;
        Ok(ParityCase {
            label: exported.label.clone(),
            project: exported.project.clone(),
            reference: stages::read(&reference_path, Stage::Slice)?,
        })
    }
}

fn preset_value(kind: &str, fields: &Map<String, Value>) -> Map<String, Value> {
    let mut preset = fields.clone();
    preset.insert("type".into(), Value::String(kind.into()));
    preset.insert("from".into(), Value::String("system".into()));
    preset.insert("instantiation".into(), Value::String("true".into()));
    preset
}
fn io(stage: Stage, error: impl std::fmt::Display) -> StageError {
    StageError::new(stage, FailureKind::Io, error)
}
fn lock(path: &Path, stage: Stage) -> Result<fs::File, StageError> {
    let file = fs::File::create(path).map_err(|error| io(stage, error))?;
    file.lock().map_err(|error| io(stage, error))?;
    Ok(file)
}
fn publish(staging: std::path::PathBuf, entry: &Path, stage: Stage) -> Result<(), StageError> {
    if entry.exists() {
        fs::remove_dir_all(entry).map_err(|error| io(stage, error))?;
    }
    fs::rename(staging, entry).map_err(|error| io(stage, error))
}
fn receipt(
    directory: &Path,
    producer: &cache_identity::Producer,
    (request, stage, request_components): (&str, Stage, &Value),
    output: &[u8],
    command_work: &Path,
) -> Result<(), StageError> {
    fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&json!({"schema":"native-cache-v2",
        "producer_digest":producer.digest,"producer":producer.receipt,"request_digest":request,"request_components":request_components,"command_work":command_work,"stage":format!("{stage:?}"),
        "output_sha256":cache_identity::raw_digest(output)})).unwrap()).map_err(|error| io(stage,error))
}
fn trace(
    work: &Path,
    producer: &cache_identity::Producer,
    request: &str,
    stage: Stage,
    outcome: &str,
) -> Result<(), StageError> {
    let evidence = tempfile::Builder::new()
        .prefix("cache-")
        .tempdir_in(work)
        .map_err(|error| io(stage, error))?
        .keep();
    fs::write(evidence.join("cache.json"),serde_json::to_vec_pretty(&json!({"schema":"native-cache-v2",
        "producer_digest":producer.digest,"producer":producer.receipt,"request_digest":request,"stage":format!("{stage:?}"),"outcome":outcome})).unwrap()).map_err(|error| io(stage,error))
}
