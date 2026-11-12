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
    pub(super) cwd: bool,
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
                    cwd: false,
                });
            }
            "--datadir" => {
                let directory = path(args.next()?);
                profile.datadir_hashes += 1;
                let digest = timed(&mut profile.datadir_hash_us, || {
                    cache_identity::directory_digest(&directory)
                })
                .ok()?;
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
    normalized.push(json!({"implicit_cwd":"native-cli-diagnostics-v1","output":outputs.len()}));
    outputs.push(Output {
        path: work.to_owned(),
        directory: true,
        cwd: true,
    });
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

pub(super) fn cwd_output(name: &std::ffi::OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    name == "result.json"
        || name.strip_suffix(".log").is_some_and(|counter| {
            counter.len() >= 5 && counter.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn eligible(args: &[String], work: &Path) -> bool {
    // Existing diagnostics may be appended to or replaced by Native, so keep them live.
    if !fs::read_dir(work).is_ok_and(|entries| {
        entries
            .into_iter()
            .all(|entry| entry.is_ok_and(|entry| !cwd_output(&entry.file_name())))
    }) {
        return false;
    }
    let mut args = args.iter();
    let mut output = false;
    let mut outputdir = false;
    let mut slice = false;
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
                outputdir |= arg == "--outputdir";
            }
            "--datadir" | "--load-settings" | "--load-filaments" | "--slice" | "--arrange"
            | "--debug" => {
                slice |= arg == "--slice";
                if args.next().is_none() {
                    return false;
                }
            }
            value if value.starts_with('-') => return false,
            _ => {}
        }
    }
    // Without outputdir Native keeps sliced G-code at PartPlate's temporary path.
    output && (!slice || outputdir)
}

fn file(path: &Path, profile: &mut Profile) -> Option<Value> {
    if !fs::metadata(path).ok()?.is_file() {
        return None;
    }
    Some(
        json!({"basename":path.file_name()?.to_str()?,"sha256":timed(&mut profile.input_hash_us, || cache_identity::file_hash(path)).ok()?}),
    )
}
