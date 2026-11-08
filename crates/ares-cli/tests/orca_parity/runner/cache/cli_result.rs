//! Bounded Native CLI results use the existing cache's locks and atomic publication.
use super::{OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, stages};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

mod success;

pub(crate) struct CliResultReference {
    pub(crate) directory: PathBuf,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) result_json: Vec<u8>,
    pub(crate) gcode: Option<Vec<u8>>,
}

#[derive(Clone, Copy)]
pub(crate) enum EarlyExit {
    InvalidValues,
    Postprocess,
    EmptyPlate,
    Validation,
    NotFound,
    ProcessNotCompatible,
    /// Absent `--load-settings` entry: the 3MF admission gate
    /// (`OrcaSlicer.cpp:1682-1690`) fires before the file reads
    /// (`:1953-2024`), so the native outcome stays `-19`.
    MissingSettingsFile,
}

/// Joined `--load-settings`/`--load-filaments` preset lists for the CLI
/// argv variant (`PrintConfig.cpp:10890-10901`); the file digests enter
/// the cache identity, not the paths.
struct LoadSettingsLists<'a> {
    settings: &'a [String],
    filaments: &'a [String],
}

/// One cached CLI-reference request: the result kind plus the optional
/// `--load-settings`/`--load-filaments` argv lists.
struct ReferenceRequest<'a> {
    kind: ResultKind,
    load_settings: Option<LoadSettingsLists<'a>>,
}

fn digest_paths(stage: Stage, paths: &[String]) -> Result<Vec<String>, StageError> {
    paths
        .iter()
        .map(|path| {
            let path = Path::new(path);
            if path.try_exists().map_err(|error| io(stage, error))? {
                Ok(cache_identity::raw_digest(&stages::read(path, stage)?))
            } else {
                // Absent entries stay argv-real but identity-stable: the
                // file name (not the tempdir path) keys the cache.
                Ok(format!(
                    "absent:{}",
                    path.file_name().unwrap().to_string_lossy()
                ))
            }
        })
        .collect()
}

impl EarlyExit {
    pub(crate) fn code(self) -> i32 {
        match self {
            Self::InvalidValues => -18,
            Self::Postprocess => -19,
            Self::EmptyPlate => -50,
            Self::Validation => -51,
            Self::NotFound => -3,
            Self::ProcessNotCompatible => -17,
            Self::MissingSettingsFile => -19,
        }
    }
}

#[derive(Clone, Copy)]
enum ResultKind {
    CustomGCode,
    CustomGCodeSuccess,
    SequentialCollision,
    SequentialLast,
    Early(EarlyExit),
    RetainedFirst,
    RetainedSecond,
}

impl ResultKind {
    fn code(self) -> i32 {
        match self {
            Self::CustomGCode => -100,
            Self::CustomGCodeSuccess => 0,
            Self::SequentialCollision => -63,
            Self::SequentialLast => 0,
            Self::Early(early) => early.code(),
            Self::RetainedFirst | Self::RetainedSecond => -102,
        }
    }

    fn schema(self) -> &'static str {
        match self {
            Self::CustomGCode => "custom-gcode-failure-v1",
            Self::CustomGCodeSuccess => "custom-gcode-success-v1",
            Self::SequentialCollision => "sequential-collision-v1",
            Self::SequentialLast => "sequential-last-v1",
            Self::Early(_) => "early-cli-failure-v1",
            Self::RetainedFirst => "retained-first-v1",
            Self::RetainedSecond => "retained-second-v1",
        }
    }

    fn native_products(self) -> &'static [&'static str] {
        match self {
            Self::CustomGCode | Self::SequentialCollision | Self::Early(_) => &["result.json"],
            Self::SequentialLast | Self::RetainedFirst | Self::CustomGCodeSuccess => {
                &["plate_1.gcode", "result.json"]
            }
            Self::RetainedSecond => &["plate_2.gcode", "result.json"],
        }
    }

    fn slice(self) -> u32 {
        match self {
            Self::RetainedSecond => 2,
            Self::CustomGCodeSuccess
            | Self::CustomGCode
            | Self::SequentialCollision
            | Self::SequentialLast
            | Self::Early(_)
            | Self::RetainedFirst => 0,
        }
    }
}

impl OrcaRunner {
    pub(crate) fn retained_cli_reference(
        &self,
        project: &Path,
        second_plate: bool,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(
            project,
            ReferenceRequest {
                kind: if second_plate {
                    ResultKind::RetainedSecond
                } else {
                    ResultKind::RetainedFirst
                },
                load_settings: None,
            },
        )
    }

    pub(crate) fn early_cli_reference(
        &self,
        project: &Path,
        rejection: EarlyExit,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(
            project,
            ReferenceRequest {
                kind: ResultKind::Early(rejection),
                load_settings: None,
            },
        )
    }

    /// The `--load-settings`/`--load-filaments` argv variant.
    pub(crate) fn load_settings_early_reference(
        &self,
        project: &Path,
        settings: &[String],
        filaments: &[String],
        rejection: EarlyExit,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(
            project,
            ReferenceRequest {
                kind: ResultKind::Early(rejection),
                load_settings: Some(LoadSettingsLists {
                    settings,
                    filaments,
                }),
            },
        )
    }

    pub(crate) fn sequential_clearance_reference(
        &self,
        project: &Path,
        last: bool,
    ) -> Result<CliResultReference, StageError> {
        self.cli_result_reference(
            project,
            ReferenceRequest {
                kind: if last {
                    ResultKind::SequentialLast
                } else {
                    ResultKind::SequentialCollision
                },
                load_settings: None,
            },
        )
    }

    fn cli_result_reference(
        &self,
        project: &Path,
        request: ReferenceRequest<'_>,
    ) -> Result<CliResultReference, StageError> {
        // Cached references cannot hide a removed mandatory producer declaration.
        super::super::environment::available()?;
        let ReferenceRequest {
            kind,
            load_settings,
        } = request;
        let stage = Stage::Slice;
        let (project, mut components) = if matches!(kind, ResultKind::Early(EarlyExit::NotFound)) {
            let project = std::path::absolute(project).map_err(|error| io(stage, error))?;
            if project.try_exists().map_err(|error| io(stage, error))? {
                return Err(io(stage, "Native missing-file input must remain absent"));
            }
            let components = json!({"input_kind":"absent", "input_path":project});
            (project, components)
        } else {
            let project = project.canonicalize().map_err(|error| io(stage, error))?;
            let components = json!({
                "input":cache_identity::raw_digest(&stages::read(&project, stage)?),
                "input_name":project.file_name(),
            });
            (project, components)
        };
        let producer = self.producer()?;
        if let Some(lists) = &load_settings {
            components["load_settings"] = json!({
                "settings": digest_paths(stage, lists.settings)?,
                "filaments": digest_paths(stage, lists.filaments)?,
            });
        }
        components["recipe"] = json!(format!(
            "{}/slice-{}/{}",
            kind.schema(),
            kind.slice(),
            kind.code()
        ));
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
            self.create_cli_result_reference(
                &project,
                ReferenceRequest {
                    kind,
                    load_settings,
                },
                &entry,
                &components,
            )?;
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
                ResultKind::SequentialLast
                | ResultKind::RetainedFirst
                | ResultKind::CustomGCodeSuccess => {
                    Some(stages::read(&entry.join("plate_1.gcode"), stage)?)
                }
                ResultKind::RetainedSecond => {
                    Some(stages::read(&entry.join("plate_2.gcode"), stage)?)
                }
                ResultKind::CustomGCode
                | ResultKind::SequentialCollision
                | ResultKind::Early(_) => None,
            },
            directory: entry,
        })
    }

    fn create_cli_result_reference(
        &self,
        project: &Path,
        request: ReferenceRequest<'_>,
        entry: &Path,
        components: &Value,
    ) -> Result<(), StageError> {
        let stage = Stage::Slice;
        let ReferenceRequest {
            kind,
            load_settings,
        } = request;
        let producer = self.producer()?;
        let request = cache_identity::digest(&[&serde_json::to_vec(components).unwrap()]);
        let staging = tempfile::Builder::new()
            .prefix("cli-result-")
            .tempdir_in(entry.parent().unwrap())
            .map_err(|error| io(stage, error))?;
        let output = staging.path().join("native");
        fs::create_dir(&output).map_err(|error| io(stage, error))?;
        let command_work = staging.path().join("command");
        fs::create_dir(&command_work).map_err(|error| io(stage, error))?;
        let mut argv = vec!["--slice".to_owned(), kind.slice().to_string()];
        if let Some(lists) = load_settings {
            argv.push("--load-settings".to_owned());
            argv.push(lists.settings.join(";"));
            argv.push("--load-filaments".to_owned());
            argv.push(lists.filaments.join(";"));
        }
        argv.push("--outputdir".to_owned());
        argv.push(output.to_string_lossy().into_owned());
        argv.push(project.to_string_lossy().into_owned());
        let outcome = stages::run(&self.bin, argv, stage, &command_work);
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
                let status: Value =
                    serde_json::from_slice(&stages::read(&evidence.join("status.json"), stage)?)
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
        let parsed: Value = serde_json::from_slice(&result).map_err(|error| io(stage, error))?;
        if parsed["return_code"] != kind.code() {
            return Err(io(stage, "Native result.json return code mismatch"));
        }
        if matches!(
            kind,
            ResultKind::SequentialLast | ResultKind::CustomGCodeSuccess
        ) {
            let gcode = stages::read(&output.join("plate_1.gcode"), stage)?;
            if gcode.is_empty() {
                return Err(io(stage, "Native last-object control emitted no G-code"));
            }
            fs::write(staging.path().join("plate_1.gcode"), gcode)
                .map_err(|error| io(stage, error))?;
        }
        if matches!(kind, ResultKind::RetainedFirst | ResultKind::RetainedSecond) {
            let filename = kind.native_products()[0];
            let gcode = stages::read(&output.join(filename), stage)?;
            if gcode.is_empty() || parsed["plate_index"] != if kind.slice() == 2 { 2 } else { 1 } {
                return Err(io(stage, "Native retained output/plate identity mismatch"));
            }
            fs::write(staging.path().join(filename), gcode).map_err(|error| io(stage, error))?;
        }
        fs::write(staging.path().join("stdout"), &captured.stdout)
            .map_err(|error| io(stage, error))?;
        fs::write(staging.path().join("stderr"), &captured.stderr)
            .map_err(|error| io(stage, error))?;
        fs::write(staging.path().join("result.json"), result).map_err(|error| io(stage, error))?;
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
        publish(staging.keep(), entry, stage)?;
        Ok(())
    }
}
