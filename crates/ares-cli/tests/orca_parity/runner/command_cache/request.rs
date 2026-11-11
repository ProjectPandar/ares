use super::{
    super::{cache_identity, stages::Stage},
    profile::{Profile, timed},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub(super) struct Output {
    pub(super) path: PathBuf,
    pub(super) directory: bool,
}

pub(super) struct Request {
    pub(super) digest: String,
    pub(super) components: Value,
    pub(super) outputs: Vec<Output>,
}

pub(super) fn normalize(
    args: &[String],
    envs: &[(&str, &str)],
    stage: Stage,
    work: &Path,
    profile: &mut Profile,
) -> Option<Request> {
    if !matches!(stage, Stage::Slice | Stage::Export) || !eligible(args, work) {
        return None;
    }
    let path = |value: &str| {
        let path = Path::new(value);
        if path.is_absolute() {
            path.to_owned()
        } else {
            work.join(path)
        }
    };
    let mut normalized = Vec::new();
    let mut outputs = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--outputdir" | "--export-3mf" => {
                let value = args.next()?;
                let target = path(value);
                let directory = arg == "--outputdir";
                // An existing output is not an input contract; keep such invocations live.
                if target.exists() && (!directory || fs::read_dir(&target).ok()?.next().is_some()) {
                    return None;
                }
                normalized.push(
                    json!({"flag":arg,"output":outputs.len(),"directory":directory,
                    "basename":if directory { None } else { Some(target.file_name()?.to_str()?) }}),
                );
                outputs.push(Output {
                    path: target,
                    directory,
                });
            }
            "--datadir" => {
                let directory = path(args.next()?);
                profile.datadir_hashes += 1;
                let (digest, reused) = timed(&mut profile.datadir_hash_us, || {
                    cache_identity::cached_directory_digest(&directory)
                })
                .ok()?;
                profile.datadir_reused |= reused;
                normalized.push(json!({"flag":arg,"tree":digest}));
            }
            "--load-settings" | "--load-filaments" => {
                let files = args
                    .next()?
                    .split(';')
                    .map(|value| file(&path(value), profile))
                    .collect::<Option<Vec<_>>>()?;
                normalized.push(json!({"flag":arg,"files":files}));
            }
            "--slice" | "--arrange" | "--debug" => {
                normalized.push(json!({"flag":arg,"value":args.next()?}));
            }
            value if value.starts_with('-') => return None,
            value => normalized.push(file(&path(value), profile)?),
        }
    }
    if outputs.is_empty() {
        return None;
    }
    let environment = timed(&mut profile.environment_us, || {
        envs.iter().copied().collect::<BTreeMap<_, _>>()
    });
    let components = json!({"schema":"native-command-v1", "args": normalized,
        "environment":environment,"env_clear":false,"stage":format!("{stage:?}")});
    let digest = cache_identity::digest(&[
        b"native-command-v1",
        &serde_json::to_vec(&components).unwrap(),
    ]);
    Some(Request {
        digest,
        components,
        outputs,
    })
}

fn eligible(args: &[String], work: &Path) -> bool {
    let mut args = args.iter();
    let mut output = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--outputdir" | "--export-3mf" => {
                let Some(value) = args.next() else {
                    return false;
                };
                let target = work.join(value);
                if target.exists()
                    && (arg != "--outputdir"
                        || !fs::read_dir(target).is_ok_and(|mut entries| entries.next().is_none()))
                {
                    return false;
                }
                output = true;
            }
            "--datadir" | "--load-settings" | "--load-filaments" | "--slice" | "--arrange"
            | "--debug" => {
                if args.next().is_none() {
                    return false;
                }
            }
            value if value.starts_with('-') => return false,
            _ => {}
        }
    }
    output
}

fn file(path: &Path, profile: &mut Profile) -> Option<Value> {
    if !fs::metadata(path).ok()?.is_file() {
        return None;
    }
    Some(
        json!({"basename":path.file_name()?.to_str()?,"sha256":timed(&mut profile.input_hash_us, || cache_identity::file_hash(path)).ok()?}),
    )
}
