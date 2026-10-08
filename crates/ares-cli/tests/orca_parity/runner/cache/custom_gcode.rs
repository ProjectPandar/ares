//! Native -100 command results use the existing cache's locks and atomic publication.
use super::{OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, stages};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(crate) struct CustomGCodeReference {
    pub(crate) directory: PathBuf,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) result_json: Vec<u8>,
}

impl OrcaRunner {
    pub(crate) fn custom_gcode_reference(
        &self,
        project: &Path,
    ) -> Result<CustomGCodeReference, StageError> {
        // Cached references cannot hide a removed mandatory producer declaration.
        super::super::environment::available()?;
        let stage = Stage::Slice;
        let project = project.canonicalize().map_err(|error| io(stage, error))?;
        let producer = self.producer()?;
        let components = json!({
            "recipe":"custom-gcode-failure-v1/slice-0/-100",
            "input":cache_identity::raw_digest(&stages::read(&project, stage)?),
            "input_name":project.file_name(),
        });
        let request = cache_identity::digest(&[&serde_json::to_vec(&components).unwrap()]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("cli-result");
        fs::create_dir_all(&parent).map_err(|error| io(stage, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), stage)?;
        if !entry.join("receipt.json").is_file() {
            let staging = tempfile::Builder::new()
                .prefix("cli-result-")
                .tempdir_in(&parent)
                .map_err(|error| io(stage, error))?;
            let output = staging.path().join("native");
            fs::create_dir(&output).map_err(|error| io(stage, error))?;
            let outcome = stages::run(
                &self.bin,
                [
                    "--slice".to_owned(),
                    "0".to_owned(),
                    "--outputdir".to_owned(),
                    output.to_string_lossy().into_owned(),
                    project.to_string_lossy().into_owned(),
                ],
                stage,
                &self.work,
            );
            let failure = match outcome {
                Err(failure) if failure.kind == stages::FailureKind::Process => failure,
                Err(failure) => return Err(failure),
                Ok(()) => return Err(io(stage, "expected Native custom G-code failure -100")),
            };
            let captured = failure
                .captured
                .as_ref()
                .ok_or_else(|| io(stage, "missing command capture"))?;
            let expected = if cfg!(unix) {
                (-100_i32).rem_euclid(256)
            } else {
                -100
            };
            if captured.code != Some(expected) {
                return Err(failure);
            }
            let files = fs::read_dir(&output)
                .map_err(|error| io(stage, error))?
                .map(|entry| entry.map(|entry| entry.file_name()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| io(stage, error))?;
            if files != [std::ffi::OsString::from("result.json")] {
                return Err(io(
                    stage,
                    format!("unexpected Native -100 product inventory: {files:?}"),
                ));
            }
            let result = stages::read(&output.join("result.json"), stage)?;
            let parsed: Value =
                serde_json::from_slice(&result).map_err(|error| io(stage, error))?;
            if parsed["return_code"] != -100 {
                return Err(io(stage, "Native result.json is not -100"));
            }
            fs::write(staging.path().join("stdout"), &captured.stdout)
                .map_err(|error| io(stage, error))?;
            fs::write(staging.path().join("stderr"), &captured.stderr)
                .map_err(|error| io(stage, error))?;
            fs::write(staging.path().join("result.json"), result)
                .map_err(|error| io(stage, error))?;
            let mut products = serde_json::Map::new();
            for file in ["stdout", "stderr", "result.json"] {
                products.insert(
                    file.into(),
                    cache_identity::raw_digest(&stages::read(&staging.path().join(file), stage)?)
                        .into(),
                );
            }
            let receipt = json!({"schema":"custom-gcode-failure-v1","request":request,
                "components":components,"producer":producer.receipt,"exit":captured.code,
                "command_evidence":failure.evidence,"outputs":products,"native_products":["result.json"]});
            fs::write(
                staging.path().join("receipt.json"),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            )
            .map_err(|error| io(stage, error))?;
            publish(staging.keep(), &entry, stage)?;
        }
        let receipt: Value =
            serde_json::from_slice(&stages::read(&entry.join("receipt.json"), stage)?)
                .map_err(|error| io(stage, error))?;
        if receipt["request"] != request || receipt["components"] != components {
            return Err(io(stage, "custom G-code cache request mismatch"));
        }
        for file in ["stdout", "stderr", "result.json"] {
            let bytes = stages::read(&entry.join(file), stage)?;
            if receipt["outputs"][file] != cache_identity::raw_digest(&bytes) {
                return Err(io(
                    stage,
                    format!("custom G-code cache output mismatch: {file}"),
                ));
            }
        }
        Ok(CustomGCodeReference {
            stdout: stages::read(&entry.join("stdout"), stage)?,
            stderr: stages::read(&entry.join("stderr"), stage)?,
            result_json: stages::read(&entry.join("result.json"), stage)?,
            directory: entry,
        })
    }
}
