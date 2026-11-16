//! Native test-adapter stages; no process behavior is added to ares-core.
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    Initialization,
    Preset,
    Export,
    Slice,
    Application,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FailureKind {
    Spawn,
    Wait,
    Timeout,
    Io,
    MissingArtifact,
    Process,
    ApplicationMismatch,
}

#[derive(Debug)]
pub(crate) struct Captured {
    pub(crate) code: Option<i32>,
    pub(crate) status: String,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct StageError {
    pub(crate) stage: Stage,
    pub(crate) kind: FailureKind,
    pub(crate) detail: String,
    pub(crate) evidence: Option<PathBuf>,
    pub(crate) captured: Option<Box<Captured>>,
}

impl StageError {
    pub(crate) fn new(stage: Stage, kind: FailureKind, detail: impl ToString) -> Self {
        Self {
            stage,
            kind,
            detail: detail.to_string(),
            evidence: None,
            captured: None,
        }
    }

    pub(crate) fn input_rejection(&self, key: &str) -> bool {
        self.kind == FailureKind::Process
            && self
                .captured
                .as_ref()
                .is_some_and(|output| attributable(output, key))
    }
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}/{:?}: {}; evidence {:?}",
            self.stage, self.kind, self.detail, self.evidence
        )
    }
}

pub(crate) fn attributable(output: &Captured, key: &str) -> bool {
    // Utils.hpp CLI codes; Unix retains the low 8 bits, Windows retains signed i32.
    let code = |value: i32| {
        if cfg!(unix) {
            value.rem_euclid(256)
        } else {
            value
        }
    };
    let text = [
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    ]
    .join("\n");
    if output.code == Some(code(-18)) {
        return text.contains("Param values in 3mf/config error:")
            && text
                .lines()
                .any(|line| line.starts_with(&format!("{key}: ")));
    }
    // Print::validate omits opt_key for -51, so each diagnostic line is
    // matched to its option key explicitly.
    if output.code == Some(code(-51)) {
        // `Print.cpp:1740-1746`: `G92 E0` in any layer-change gcode is
        // rejected when `use_relative_e_distances` is off (absolute E).
        if key == "use_relative_e_distances"
            && text
                .lines()
                .any(|line| line.ends_with("incompatible with absolute extruder addressing."))
        {
            return true;
        }
        // `Print.cpp:1389`: a schema-legal `printable_height` below the
        // model height is a feasibility rejection, not a value error.
        if key == "printable_height"
            && text
                .lines()
                .any(|line| line.ends_with("exceeds the maximum build volume height."))
        {
            return true;
        }
        // `Print.cpp:1654/1659`: a schema-legal layer height above the
        // nozzle diameter is a feasibility rejection, not a value error.
        if matches!(key, "layer_height" | "initial_layer_print_height")
            && text
                .lines()
                .any(|line| line == "Layer height cannot exceed nozzle diameter.")
        {
            return true;
        }
        // `Print.cpp:1549/1687`: a line-width percent that resolves below the
        // extrusion minimum. Both skin/skeleton infill widths share the
        // generic diagnostic (the opt_key is not echoed in the -51 text).
        if matches!(key, "skin_infill_line_width" | "skeleton_infill_line_width")
            && text.lines().any(|line| line == "Line width too small")
        {
            return true;
        }
        // `bridge_line_width`: the unique line-width-vs-nozzle diagnostic.
        return key == "bridge_line_width"
            && text
                .lines()
                .any(|line| line == "Bridge line width must not exceed nozzle diameter");
    }
    // `CLI_GCODE_PATH_IN_UNPRINTABLE_AREA` (-102, `Utils.hpp:74`): smooth
    // timelapse forces a prime tower (`Print.cpp:3317-3318`), and on
    // machines whose usable bed cannot host the generated tower Orca
    // rejects the run after slicing — a combination rejection, not a
    // value error in `timelapse_type` itself.
    if output.code == Some(code(-102)) {
        return key == "timelapse_type";
    }
    false
}

pub(crate) fn run(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    stage: Stage,
    work: &Path,
) -> Result<(), StageError> {
    run_with_env(bin, args, &[], stage, work)
}

pub(crate) fn run_live(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    stage: Stage,
    work: &Path,
) -> Result<(), StageError> {
    run_with_timeout(bin, args, stage, work, Duration::from_secs(300))
}

pub(crate) fn run_with_timeout(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    stage: Stage,
    work: &Path,
    timeout: Duration,
) -> Result<(), StageError> {
    super::command::run(
        bin,
        args,
        &[],
        super::command::Limits {
            stage,
            work,
            timeout,
        },
    )
}

/// `run` with `envs` set on the child only (the parent environment is shared by parallel tests).
pub(crate) fn run_with_env(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    envs: &[(&str, &str)],
    stage: Stage,
    work: &Path,
) -> Result<(), StageError> {
    super::command::run_cached(
        bin,
        args,
        envs,
        super::command::Limits {
            stage,
            work,
            timeout: Duration::from_secs(300),
        },
    )
}

pub(crate) fn run_live_with_env(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    envs: &[(&str, &str)],
    stage: Stage,
    work: &Path,
) -> Result<(), StageError> {
    let timeout = Duration::from_secs(300);
    super::command::run(
        bin,
        args,
        envs,
        super::command::Limits {
            stage,
            work,
            timeout,
        },
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn run_exact_with_env(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    envs: &[(&str, &str)],
    stage: Stage,
    work: &Path,
) -> Result<(), StageError> {
    super::command::run_exact(
        bin,
        args,
        envs,
        super::command::Limits {
            stage,
            work,
            timeout: Duration::from_secs(300),
        },
    )
}

pub(super) fn capture_error(failure: &mut Option<(FailureKind, String)>, detail: String) {
    match failure {
        Some((FailureKind::Timeout | FailureKind::Wait, primary)) => {
            primary.push_str(&format!("; capture I/O error: {detail}"));
        }
        _ => *failure = Some((FailureKind::Io, detail)),
    }
}

pub(super) fn finish(
    directory: &Path,
    stage: Stage,
    mut failure: Option<(FailureKind, String)>,
    captured: Captured,
) -> Result<(), StageError> {
    for (name, bytes) in [
        ("stdout.bin", &captured.stdout),
        ("stderr.bin", &captured.stderr),
    ] {
        if let Err(error) = crate::artifacts::write(&directory.join(name), bytes) {
            capture_error(&mut failure, error);
        }
    }
    let hash = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let status = serde_json::json!({
        "code": captured.code, "native_status": captured.status,
        "failure": failure.as_ref().map(|(kind, detail)| format!("{kind:?}: {detail}")),
        "snapshot_scope": "direct child output only; not process-tree completion",
        "snapshot_incomplete": failure.as_ref().is_some_and(|(kind, _)| *kind != FailureKind::Process),
        "live_captures": "stdout.live/stderr.live are non-authoritative and may continue changing after return",
        "stdout": {"file": "stdout.bin", "bytes": captured.stdout.len(), "sha256": hash(&captured.stdout)},
        "stderr": {"file": "stderr.bin", "bytes": captured.stderr.len(), "sha256": hash(&captured.stderr)},
    });
    if let Err(error) = crate::artifacts::write(
        &directory.join("status.json"),
        &serde_json::to_vec_pretty(&status).unwrap(),
    ) {
        capture_error(&mut failure, error);
    }
    if let Some((kind, detail)) = failure {
        Err(StageError {
            stage,
            kind,
            detail,
            evidence: Some(directory.to_owned()),
            captured: Some(Box::new(captured)),
        })
    } else {
        Ok(())
    }
}

pub(crate) fn read(path: &Path, stage: Stage) -> Result<Vec<u8>, StageError> {
    std::fs::read(path).map_err(|error| {
        StageError::new(
            stage,
            if error.kind() == std::io::ErrorKind::NotFound {
                FailureKind::MissingArtifact
            } else {
                FailureKind::Io
            },
            format!("{path:?}: {error}"),
        )
    })
}
