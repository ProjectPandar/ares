use super::{
    CaseInputs, ExportedCase, OrcaRunner, ParityCase, PresetKind, apply_override, cache_identity,
    digest_slug, join_paths,
    stages::{self, FailureKind, Stage, StageError},
    write_preset,
};
use serde_json::{Map, Value, json};
use std::{fs, io::Read as _, path::Path};

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
            stages::run(
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
            stages::run(
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

/// Rewrite the exported project so the i-th object prints with
/// `object_filaments[i]` (1-based extruder), and expand single-entry
/// `filament_*` lists to the filament count. The native export composes
/// every object with `extruder=1` and collapses identical per-filament
/// values to one entry; the UI saves N entries for every filament option,
/// and the G-code generation reads per-filament vectors by index (a
/// 1-entry list for a 2-filament project crashes it). Both patches live
/// in `Metadata/model_settings.config` and `Metadata/project_settings.config`,
/// which both slicers read from the 3mf.
fn assign_object_filaments(
    project: &[u8],
    object_filaments: &[u32],
) -> Result<Vec<u8>, StageError> {
    if object_filaments.is_empty() {
        return Ok(project.to_vec());
    }
    let settings_name = "Metadata/model_settings.config";
    let project_settings_name = "Metadata/project_settings.config";
    let mut source = zip::ZipArchive::new(std::io::Cursor::new(project))
        .map_err(|error| io(Stage::Export, error))?;
    let mut settings = String::new();
    source
        .by_name(settings_name)
        .map_err(|error| io(Stage::Export, error))?
        .read_to_string(&mut settings)
        .map_err(|error| io(Stage::Export, error))?;
    let patched = patch_extruder_metadata(&settings, object_filaments);
    let mut project_settings = String::new();
    source
        .by_name(project_settings_name)
        .map_err(|error| io(Stage::Export, error))?
        .read_to_string(&mut project_settings)
        .map_err(|error| io(Stage::Export, error))?;
    let expanded = expand_filament_lists(&project_settings);
    let mut output = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut output);
        let options: zip::write::SimpleFileOptions = Default::default();
        for index in 0..source.len() {
            let mut member = source
                .by_index(index)
                .map_err(|error| io(Stage::Export, error))?;
            if member.name() == settings_name {
                writer
                    .start_file(settings_name, options)
                    .map_err(|error| io(Stage::Export, error))?;
                std::io::Write::write_all(&mut writer, patched.as_bytes())
                    .map_err(|error| io(Stage::Export, error))?;
            } else if member.name() == project_settings_name {
                writer
                    .start_file(project_settings_name, options)
                    .map_err(|error| io(Stage::Export, error))?;
                std::io::Write::write_all(&mut writer, expanded.as_bytes())
                    .map_err(|error| io(Stage::Export, error))?;
            } else {
                let mut bytes = Vec::new();
                member
                    .read_to_end(&mut bytes)
                    .map_err(|error| io(Stage::Export, error))?;
                writer
                    .start_file(member.name().to_owned(), options)
                    .map_err(|error| io(Stage::Export, error))?;
                std::io::Write::write_all(&mut writer, &bytes)
                    .map_err(|error| io(Stage::Export, error))?;
            }
        }
        writer.finish().map_err(|error| io(Stage::Export, error))?;
    }
    Ok(output.into_inner())
}

/// Replace the `extruder` metadata value inside each `<object>` block, in
/// document order.
fn patch_extruder_metadata(settings: &str, object_filaments: &[u32]) -> String {
    let mut result = String::with_capacity(settings.len());
    let mut tail = settings;
    let mut index = 0usize;
    while let Some(object_start) = tail.find("<object ") {
        let Some(close_rel) = tail[object_start..].find("</object>") else {
            break;
        };
        let block_end = object_start + close_rel + "</object>".len();
        result.push_str(&tail[..object_start]);
        let block = &tail[object_start..block_end];
        let patched = object_filaments
            .get(index)
            .and_then(|filament| set_extruder(block, *filament))
            .unwrap_or_else(|| block.to_owned());
        result.push_str(&patched);
        index += 1;
        tail = &tail[block_end..];
    }
    result.push_str(tail);
    result
}

fn set_extruder(block: &str, filament: u32) -> Option<String> {
    let key = "<metadata key=\"extruder\" value=\"";
    let position = block.find(key)?;
    let value_start = position + key.len();
    let length = block[value_start..].find('"')?;
    let mut patched = block.to_string();
    patched.replace_range(value_start..value_start + length, &filament.to_string());
    Some(patched)
}

/// Expand every single-entry `filament_*` list to the filament count (the
/// count of `filament_colour`), matching the shape the UI saves for a
/// multi-filament project.
fn expand_filament_lists(settings: &str) -> String {
    let mut value: serde_json::Value = match serde_json::from_str(settings) {
        Ok(value) => value,
        Err(_) => return settings.to_owned(),
    };
    let Some(map) = value.as_object_mut() else {
        return settings.to_owned();
    };
    let Some(count) = map
        .get("filament_colour")
        .and_then(Value::as_array)
        .map(Vec::len)
    else {
        return settings.to_owned();
    };
    if count < 2 {
        return settings.to_owned();
    }
    for (key, entry) in map.iter_mut() {
        if !key.starts_with("filament_") {
            continue;
        }
        let Some(list) = entry.as_array_mut() else {
            continue;
        };
        while !list.is_empty() && list.len() < count {
            let last = list.last().cloned();
            if let Some(last) = last {
                list.push(last);
            }
        }
    }
    serde_json::to_string(&value).unwrap_or_else(|_| settings.to_owned())
}
