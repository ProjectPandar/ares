//! Bounded Native CLI results use the existing cache's locks and atomic publication.
use super::{OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, stages};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(crate) struct CliResultReference {
    pub(crate) directory: PathBuf,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) result_json: Vec<u8>,
    pub(crate) gcode: Option<Vec<u8>>,
}

#[derive(Clone, Copy)]
enum ResultKind {
    CustomGCode,
    SequentialCollision,
    SequentialLast,
}

impl ResultKind {
    fn code(self) -> i32 {
        match self {
            Self::CustomGCode => -100,
            Self::SequentialCollision => -63,
            Self::SequentialLast => 0,
        }
    }

    fn schema(self) -> &'static str {
        match self {
            Self::CustomGCode => "custom-gcode-failure-v1",
            Self::SequentialCollision => "sequential-collision-v1",
            Self::SequentialLast => "sequential-last-v1",
        }
    }

    fn native_products(self) -> &'static [&'static str] {
        match self {
            Self::CustomGCode | Self::SequentialCollision => &["result.json"],
            Self::SequentialLast => &["plate_1.gcode", "result.json"],
        }
    }
}

impl OrcaRunner {
    pub(crate) fn custom_gcode_reference(
        &self,
        project: &Path,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(project, ResultKind::CustomGCode)
    }

    pub(crate) fn sequential_clearance_reference(
        &self,
        project: &Path,
        last: bool,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(
            project,
            if last {
                ResultKind::SequentialLast
            } else {
                ResultKind::SequentialCollision
            },
        )
    }

    fn cli_result_reference(
        &self,
        project: &Path,
        kind: ResultKind,
    ) -> Result<CliResultReference, StageError> {
        // Cached references cannot hide a removed mandatory producer declaration.
        super::super::environment::available()?;
        let stage = Stage::Slice;
        let project = project.canonicalize().map_err(|error| io(stage, error))?;
        let producer = self.producer()?;
        let components = json!({
            "recipe":format!("{}/slice-0/{}", kind.schema(), kind.code()),
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
            let command_work = staging.path().join("command");
            fs::create_dir(&command_work).map_err(|error| io(stage, error))?;
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
                &command_work,
            );
            let expected = if cfg!(unix) {
                kind.code().rem_euclid(256)
            } else {
                kind.code()
            };
            let (captured, evidence) = match outcome {
                Err(failure) if failure.kind == stages::FailureKind::Process => {
                    if kind.code() == 0
                        || failure.captured.as_ref().map(|c| c.code) != Some(Some(expected))
                    {
                        return Err(failure);
                    }
                    (*failure.captured.unwrap(), failure.evidence.unwrap())
                }
                Err(failure) => return Err(failure),
                Ok(()) if kind.code() == 0 => {
                    // The isolated work directory contains exactly this successful command.
                    let evidence = fs::read_dir(&command_work)
                        .map_err(|error| io(stage, error))?
                        .next()
                        .unwrap()
                        .map_err(|error| io(stage, error))?
                        .path();
                    let status: Value = serde_json::from_slice(&stages::read(
                        &evidence.join("status.json"),
                        stage,
                    )?)
                    .map_err(|error| io(stage, error))?;
                    if status["code"] != 0 || status["snapshot_incomplete"] != false {
                        return Err(io(stage, "incomplete Native successful command capture"));
                    }
                    (
                        stages::Captured {
                            code: Some(0),
                            status: status["native_status"].to_string(),
                            stdout: stages::read(&evidence.join("stdout.bin"), stage)?,
                            stderr: stages::read(&evidence.join("stderr.bin"), stage)?,
                        },
                        evidence,
                    )
                }
                Ok(()) => {
                    return Err(io(
                        stage,
                        format!("expected Native CLI failure {}", kind.code()),
                    ));
                }
            };
            let mut files = fs::read_dir(&output)
                .map_err(|error| io(stage, error))?
                .map(|entry| entry.map(|entry| entry.file_name()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| io(stage, error))?;
            files.sort();
            if files
                != kind
                    .native_products()
                    .iter()
                    .map(std::ffi::OsString::from)
                    .collect::<Vec<_>>()
            {
                return Err(io(
                    stage,
                    format!(
                        "unexpected Native {} product inventory: {files:?}",
                        kind.code()
                    ),
                ));
            }
            let result = stages::read(&output.join("result.json"), stage)?;
            let parsed: Value =
                serde_json::from_slice(&result).map_err(|error| io(stage, error))?;
            if parsed["return_code"] != kind.code() {
                return Err(io(stage, "Native result.json return code mismatch"));
            }
            if let ResultKind::SequentialLast = kind {
                let gcode = stages::read(&output.join("plate_1.gcode"), stage)?;
                if gcode.is_empty() {
                    return Err(io(stage, "Native last-object control emitted no G-code"));
                }
                fs::write(staging.path().join("plate_1.gcode"), gcode)
                    .map_err(|error| io(stage, error))?;
            }
            fs::write(staging.path().join("stdout"), &captured.stdout)
                .map_err(|error| io(stage, error))?;
            fs::write(staging.path().join("stderr"), &captured.stderr)
                .map_err(|error| io(stage, error))?;
            fs::write(staging.path().join("result.json"), result)
                .map_err(|error| io(stage, error))?;
            let mut products = serde_json::Map::new();
            for file in ["stdout", "stderr"]
                .into_iter()
                .chain(kind.native_products().iter().copied())
            {
                products.insert(
                    file.into(),
                    cache_identity::raw_digest(&stages::read(&staging.path().join(file), stage)?)
                        .into(),
                );
            }
            // Paths in command evidence must refer to the published directory, not vanished staging.
            let published_evidence = entry.join(evidence.strip_prefix(staging.path()).unwrap());
            let receipt = json!({"schema":kind.schema(),"request":request,
                "components":components,"producer":producer.receipt,"exit":captured.code,
                "command_evidence":published_evidence,"outputs":products,"native_products":kind.native_products()});
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
            return Err(io(stage, "Native CLI result cache request mismatch"));
        }
        for file in ["stdout", "stderr"]
            .into_iter()
            .chain(kind.native_products().iter().copied())
        {
            let bytes = stages::read(&entry.join(file), stage)?;
            if receipt["outputs"][file] != cache_identity::raw_digest(&bytes) {
                return Err(io(
                    stage,
                    format!("Native CLI result cache output mismatch: {file}"),
                ));
            }
        }
        Ok(CliResultReference {
            stdout: stages::read(&entry.join("stdout"), stage)?,
            stderr: stages::read(&entry.join("stderr"), stage)?,
            result_json: stages::read(&entry.join("result.json"), stage)?,
            gcode: match kind {
                ResultKind::SequentialLast => {
                    Some(stages::read(&entry.join("plate_1.gcode"), stage)?)
                }
                ResultKind::CustomGCode | ResultKind::SequentialCollision => None,
            },
            directory: entry,
        })
    }
}
