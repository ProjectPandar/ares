#[path = "command_cache/request.rs"]
mod request;
#[path = "command_cache/storage.rs"]
mod storage;

use super::{
    cache_identity,
    command::{self, Limits},
    stages::{self, Captured, FailureKind, StageError},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

pub(super) fn run(
    bin: &Path,
    args: Vec<String>,
    envs: &[(&str, &str)],
    limits: Limits<'_>,
) -> Result<(), StageError> {
    let started = Instant::now();
    let Some(request) = request::normalize(&args, envs, limits.stage, limits.work) else {
        return command::run_exact(bin, args, envs, limits);
    };
    // Instrumented helpers have their own producer contracts and stay with their owning cache.
    let declared = std::env::var_os("ARES_ORCA_BIN").and_then(|path| fs::canonicalize(path).ok());
    if declared.as_deref() != fs::canonicalize(bin).ok().as_deref() || declared.is_none() {
        return command::run_exact(bin, args, envs, limits);
    }
    let producer = match producer(bin) {
        Ok(producer) => producer,
        Err(error) => {
            trace(
                limits.work,
                json!({"outcome":"bypass-producer","detail":error.to_string()}),
            );
            return command::run_exact(bin, args, envs, limits);
        }
    };
    let root = std::env::var_os("ARES_ORCA_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| limits.work.to_owned());
    let parent = root
        .join("native-cache-v2")
        .join(&producer.digest)
        .join("command");
    let entry = parent.join(&request.digest);
    let lock = (|| -> std::io::Result<fs::File> {
        fs::create_dir_all(&parent)?;
        let lock = fs::File::create(entry.with_extension("lock"))?;
        lock.lock()?;
        Ok(lock)
    })();
    let Ok(_lock) = lock else {
        return command::run_exact(bin, args, envs, limits);
    };
    if let Some((packet, captured)) = storage::load(&entry, &request, &producer.digest)
        && ((captured.code == Some(0) && packet.rejection.is_none())
            || (packet.rejection.is_some() && deterministic_rejection(&captured)))
        && storage::materialize(&entry, &request, &packet, limits.work)
            .map_err(|error| StageError::new(limits.stage, FailureKind::Io, error))?
    {
        let evidence = tempfile::Builder::new()
            .prefix("command-cache-hit-")
            .tempdir_in(limits.work)
            .map_err(|error| StageError::new(limits.stage, FailureKind::Io, error))?
            .keep();
        fs::write(evidence.join("command.json"), serde_json::to_vec_pretty(&json!({
            "binary":bin,"args":args,"stage":format!("{:?}",limits.stage),"set_environment":envs,
            "cwd":limits.work,"test_id":std::env::args().skip(1).find(|arg| arg.contains("::")),
            "cache_hit":true,"source":entry,"native_executed":false,
        })).unwrap()).map_err(|error| StageError::new(limits.stage, FailureKind::Io, error))?;
        trace(
            &evidence,
            json!({"outcome":"hit","producer_digest":producer.digest,"request_digest":request.digest,"elapsed_us":started.elapsed().as_micros(),"native_executed":false}),
        );
        return stages::finish(
            &evidence,
            limits.stage,
            packet
                .rejection
                .map(|detail| (FailureKind::Process, detail)),
            captured,
        );
    }
    let stage = limits.stage;
    let work = limits.work;
    let result = command::run_recorded(bin, args, envs, limits);
    let captured = match &result {
        Ok(directory) => read_capture(directory),
        Err(error) if error.kind == FailureKind::Process => error
            .captured
            .as_ref()
            .filter(|output| deterministic_rejection(output))
            .map(|output| Captured {
                code: output.code,
                status: output.status.clone(),
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            }),
        Err(_) => None,
    };
    let mut publication = None;
    if let Some(captured) = captured {
        let publish = (|| -> std::io::Result<()> {
            let staging = tempfile::Builder::new()
                .prefix("command-")
                .tempdir_in(&parent)?;
            storage::store(
                staging.path(),
                &request,
                &producer.digest,
                &captured,
                result.as_ref().err().map(|error| error.detail.clone()),
            )?;
            if entry.exists() {
                fs::remove_dir_all(&entry)?;
            }
            fs::rename(staging.path(), &entry)
        })();
        publication = Some(publish.err().map(|error| error.to_string()));
    }
    trace(
        work,
        json!({"outcome":"miss","stage":format!("{stage:?}"),"producer_digest":producer.digest,"request_digest":request.digest,
        "elapsed_us":started.elapsed().as_micros(),"native_executed":true,"publication":publication}),
    );
    result.map(|_| ())
}

fn producer(bin: &Path) -> Result<Arc<cache_identity::Producer>, StageError> {
    static PRODUCERS: OnceLock<Mutex<HashMap<PathBuf, Arc<cache_identity::Producer>>>> =
        OnceLock::new();
    let mut producers = PRODUCERS.get_or_init(Mutex::default).lock().unwrap();
    if let Some(producer) = producers.get(bin) {
        return Ok(producer.clone());
    }
    let producer = Arc::new(cache_identity::identify(bin)?);
    producers.insert(bin.to_owned(), producer.clone());
    Ok(producer)
}

fn read_capture(directory: &Path) -> Option<Captured> {
    let status: Value =
        serde_json::from_slice(&fs::read(directory.join("status.json")).ok()?).ok()?;
    Some(Captured {
        code: Some(status["code"].as_i64()? as i32),
        status: status["native_status"].as_str()?.into(),
        stdout: fs::read(directory.join("stdout.bin")).ok()?,
        stderr: fs::read(directory.join("stderr.bin")).ok()?,
    })
}

fn deterministic_rejection(captured: &Captured) -> bool {
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&captured.stdout),
        String::from_utf8_lossy(&captured.stderr)
    );
    let explicit = text
        .lines()
        .filter_map(|line| line.split_once(": ").map(|(key, _)| key))
        .filter(|key| ares_core::option_definition(key).is_some());
    explicit
        .chain([
            "use_relative_e_distances",
            "printable_height",
            "layer_height",
            "initial_layer_print_height",
            "skin_infill_line_width",
            "skeleton_infill_line_width",
            "bridge_line_width",
            "timelapse_type",
        ])
        .any(|key| stages::attributable(captured, key))
}

fn trace(work: &Path, value: Value) {
    if let Ok(directory) = tempfile::Builder::new()
        .prefix("cache-command-")
        .tempdir_in(work)
    {
        let _ = fs::write(
            directory.keep().join("cache.json"),
            serde_json::to_vec_pretty(&value).unwrap(),
        );
    }
}
