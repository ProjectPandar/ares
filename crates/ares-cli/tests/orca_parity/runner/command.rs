//! Portable file-backed command capture: never wait for inherited-handle EOF.
use super::stages::{Captured, FailureKind, Stage, StageError, capture_error, finish};
use std::{
    fs::File,
    io::Read,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

/// Where and how long one command runs.
pub(super) struct Limits<'a> {
    pub(super) stage: Stage,
    pub(super) work: &'a Path,
    pub(super) timeout: Duration,
}

pub(super) fn run(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    envs: &[(&str, &str)],
    limits: Limits<'_>,
) -> Result<(), StageError> {
    let args = super::application::native_arguments(args.into_iter().collect())
        .map_err(|error| StageError::new(limits.stage, FailureKind::Io, error))?;
    run_exact(bin, args, envs, limits)
}

pub(super) fn run_exact(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    envs: &[(&str, &str)],
    Limits {
        stage,
        work,
        timeout,
    }: Limits,
) -> Result<(), StageError> {
    let started = Instant::now();
    let mut timings = serde_json::json!({"clock": "monotonic", "unit": "microseconds"});
    let io_error = |error| StageError::new(stage, FailureKind::Io, error);
    let requested_bin = bin;
    let bin = std::path::absolute(bin).map_err(io_error)?;
    let directory = tempfile::Builder::new()
        .prefix("command-")
        .tempdir_in(work)
        .map_err(io_error)?
        .keep();
    let args: Vec<String> = args.into_iter().collect();
    let environment = [
        "ORCA_APPDIR",
        "ORCA_LIB_CACHE",
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "LC_ALL",
    ]
    .into_iter()
    .map(|key| {
        (
            key,
            std::env::var_os(key).map(|value| value.to_string_lossy().into_owned()),
        )
    })
    .collect::<std::collections::BTreeMap<_, _>>();
    std::fs::write(directory.join("command.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "binary": bin, "requested_binary": requested_bin, "args": args, "stage": format!("{stage:?}"),
        "cwd": work, "timeout_millis": timeout.as_millis(), "shutdown_grace_millis": 2000,
        "environment": environment, "set_environment": envs,
        "capture_policy": "file-backed; immutable length-bounded snapshots; no inherited-pipe EOF wait",
        "process_tree_termination": "not guaranteed",
    })).unwrap()).map_err(io_error)?;
    let stdout_path = directory.join("stdout.live");
    let stderr_path = directory.join("stderr.live");
    let stdout_file = File::create(&stdout_path).map_err(io_error)?;
    let stderr_file = File::create(&stderr_path).map_err(io_error)?;
    timings["prepared_at_us"] = serde_json::json!(started.elapsed().as_micros());
    let spawn_started = Instant::now();
    let spawned = Command::new(&bin)
        .args(&args)
        .envs(envs.iter().copied())
        .current_dir(work)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn();
    timings["spawn_us"] = serde_json::json!(spawn_started.elapsed().as_micros());
    timings["spawn_returned_at_us"] = serde_json::json!(started.elapsed().as_micros());
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            return timed_finish(
                &directory,
                stage,
                Some((FailureKind::Spawn, error.to_string())),
                Captured {
                    code: None,
                    status: "not spawned".into(),
                    stdout: vec![],
                    stderr: vec![],
                },
                (timings, started),
            );
        }
    };
    let pid = child.id();
    let deadline = Instant::now() + timeout;
    let mut failure = None;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => {
                let reason = match result {
                    Err(error) => (FailureKind::Wait, error.to_string()),
                    Ok(_) => (FailureKind::Timeout, "orca-slicer timed out".into()),
                };
                timings["failure_detected_at_us"] =
                    serde_json::json!(started.elapsed().as_micros());
                let (status, shutdown, shutdown_timings) = shutdown(&mut child);
                timings["shutdown"] = shutdown_timings;
                failure = Some((reason.0, format!("{}; {shutdown}", reason.1)));
                break status;
            }
        }
    };
    timings["child_wait_completed_at_us"] = serde_json::json!(started.elapsed().as_micros());
    if failure.is_none() && status.is_some_and(|status| !status.success()) {
        failure = Some((FailureKind::Process, "unsuccessful Orca process".into()));
    }
    let lifecycle_started = Instant::now();
    let lifecycle = serde_json::json!({"direct_child_pid": pid, "direct_child_reaped": status.is_some(),
        "native_status": status.map(|status| status.to_string()), "process_tree_termination": "not guaranteed"});
    if let Err(error) = std::fs::write(
        directory.join("lifecycle.json"),
        serde_json::to_vec_pretty(&lifecycle).unwrap(),
    ) {
        capture_error(&mut failure, error.to_string());
    }
    timings["lifecycle_persist_us"] = serde_json::json!(lifecycle_started.elapsed().as_micros());
    let snapshot_started = Instant::now();
    let (stdout, stdout_error) = snapshot(&stdout_path);
    timings["stdout_snapshot_us"] = serde_json::json!(snapshot_started.elapsed().as_micros());
    let snapshot_started = Instant::now();
    let (stderr, stderr_error) = snapshot(&stderr_path);
    timings["stderr_snapshot_us"] = serde_json::json!(snapshot_started.elapsed().as_micros());
    for error in [stdout_error, stderr_error].into_iter().flatten() {
        capture_error(&mut failure, error);
    }
    timings["snapshots_completed_at_us"] = serde_json::json!(started.elapsed().as_micros());
    timed_finish(
        &directory,
        stage,
        failure,
        Captured {
            code: status.and_then(|status| status.code()),
            status: status.map_or_else(|| "unavailable".into(), |status| status.to_string()),
            stdout,
            stderr,
        },
        (timings, started),
    )
}

fn timed_finish(
    directory: &Path,
    stage: Stage,
    failure: Option<(FailureKind, String)>,
    captured: Captured,
    (mut timings, started): (serde_json::Value, Instant),
) -> Result<(), StageError> {
    let persist_started = Instant::now();
    let mut result = finish(directory, stage, failure, captured);
    timings["finish_persist_us"] = serde_json::json!(persist_started.elapsed().as_micros());
    timings["finish_completed_at_us"] = serde_json::json!(started.elapsed().as_micros());
    // The final timing-file write cannot measure its own duration. All other
    // synchronous persistence, including immutable captures/status, is covered.
    if let Err(error) = std::fs::write(
        directory.join("timings.json"),
        serde_json::to_vec_pretty(&timings).unwrap(),
    ) {
        let detail = format!("timing evidence I/O error: {error}");
        match &mut result {
            Err(primary) => primary.detail.push_str(&format!("; {detail}")),
            Ok(()) => {
                let mut error = StageError::new(stage, FailureKind::Io, detail);
                error.evidence = Some(directory.to_owned());
                result = Err(error);
            }
        }
    }
    result
}

fn shutdown(child: &mut Child) -> (Option<ExitStatus>, String, serde_json::Value) {
    let kill_started = Instant::now();
    let kill = child.kill();
    let kill_us = kill_started.elapsed().as_micros();
    let reap_started = Instant::now();
    let deadline = Instant::now() + Duration::from_secs(2);
    let reap = loop {
        match child.try_wait() {
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => break result,
        }
    };
    let detail = format!("kill={kill:?}; reap={reap:?}");
    // None/error is an explicitly unreaped child, never a successful stage.
    (
        reap.ok().flatten(),
        detail,
        serde_json::json!({"kill_us": kill_us, "reap_us": reap_started.elapsed().as_micros()}),
    )
}

fn snapshot(path: &Path) -> (Vec<u8>, Option<String>) {
    let mut bytes = Vec::new();
    let result = (|| -> std::io::Result<()> {
        let file = File::open(path)?;
        let length = file.metadata()?.len();
        file.take(length).read_to_end(&mut bytes)?;
        Ok(())
    })();
    (
        bytes,
        result
            .err()
            .map(|error| format!("snapshot {path:?}: {error}")),
    )
}
