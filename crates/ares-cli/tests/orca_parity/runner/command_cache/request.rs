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
    pub(super) isolated_cwd: bool,
}

fn cli_option_flag(arg: &str) -> Option<(&str, Option<&str>)> {
    use ares_core::OptionValueKind as Kind;
    let rest = arg.strip_prefix("--")?;
    let (name, inline) = match rest.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (rest, None),
    };
    let definition = ares_core::option_definition(&name.replace('-', "_"))?;
    match definition.kind {
        Kind::Float
        | Kind::FloatOrPercent
        | Kind::FloatsOrPercents
        | Kind::Percent
        | Kind::Percents
        | Kind::PercentsNullable
        | Kind::Int
        | Kind::Ints
        | Kind::IntsNullable
        | Kind::Floats
        | Kind::FloatsNullable
        | Kind::Enum
        | Kind::Enums
        | Kind::EnumsNullable => Some((name, inline)),
        // Bare boolean flags may toggle without consuming a value; only cache explicit `=value`.
        Kind::Bool | Kind::Bools | Kind::BoolsNullable if inline.is_some() => Some((name, inline)),
        // Bypass bare bools, string/path/script options (e.g. post_process, bed_custom_*), and geometry lists.
        Kind::Bool
        | Kind::Bools
        | Kind::BoolsNullable
        | Kind::String
        | Kind::Strings
        | Kind::Point
        | Kind::Points
        | Kind::PointsGroups => None,
    }
}

pub(super) fn normalize(
    args: &[String],
    envs: &[(&str, &str)],
    stage: Stage,
    work: &Path,
    profile: &mut Profile,
) -> Option<Request> {
    if !matches!(stage, Stage::Slice | Stage::Export) {
        return None;
    }
    let isolated_cwd = eligible(args, work)?;
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
                let digest = datadir_digest(&directory, profile)?;
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
            value if value.starts_with('-') => {
                let (name, inline) = cli_option_flag(value)?;
                let option_value = match inline {
                    Some(inline) => inline,
                    None => args.next()?.as_str(),
                };
                normalized.push(json!({"flag": format!("--{name}"), "value": option_value}));
            }
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
        isolated_cwd,
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

fn eligible(args: &[String], work: &Path) -> Option<bool> {
    let entries = fs::read_dir(work).ok()?;
    let mut has_cwd = false;
    for entry in entries {
        let entry = entry.ok()?;
        if cwd_output(&entry.file_name()) {
            has_cwd = true;
        }
    }
    let mut args = args.iter();
    let mut output = false;
    let mut outputdir = false;
    let mut slice = false;
    let mut all_paths_absolute = true;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--outputdir" | "--export-3mf" => {
                let value = args.next()?;
                all_paths_absolute &= Path::new(value).is_absolute();
                let target = work.join(value);
                if target.exists()
                    && (arg != "--outputdir"
                        || !fs::read_dir(target).is_ok_and(|mut entries| entries.next().is_none()))
                {
                    return None;
                }
                output = true;
                outputdir |= arg == "--outputdir";
            }
            "--datadir" => {
                let value = args.next()?;
                all_paths_absolute &= Path::new(value).is_absolute();
            }
            "--load-settings" | "--load-filaments" => {
                let value = args.next()?;
                all_paths_absolute &= value.split(';').all(|item| Path::new(item).is_absolute());
            }
            "--slice" | "--arrange" | "--debug" => {
                slice |= arg == "--slice";
                args.next()?;
            }
            value if value.starts_with('-') => {
                let (_, inline) = cli_option_flag(value)?;
                if inline.is_none() {
                    args.next()?;
                }
            }
            value => {
                all_paths_absolute &= Path::new(value).is_absolute();
            }
        }
    }
    // Without outputdir Native keeps sliced G-code at PartPlate's temporary path.
    if !output || (slice && !outputdir) {
        return None;
    }
    // Existing diagnostics in work require explicit --outputdir and absolute paths for isolated cwd.
    if has_cwd && (!outputdir || !all_paths_absolute) {
        return None;
    }
    Some(has_cwd)
}

fn datadir_digest(directory: &Path, profile: &mut Profile) -> Option<String> {
    if let Some(result) = super::producer::immutable_datadir_digest(directory) {
        return result.ok();
    }
    profile.datadir_hashes += 1;
    timed(&mut profile.datadir_hash_us, || {
        cache_identity::directory_digest(directory)
    })
    .ok()
}

fn file(path: &Path, profile: &mut Profile) -> Option<Value> {
    if !fs::metadata(path).ok()?.is_file() {
        return None;
    }
    Some(
        json!({"basename":path.file_name()?.to_str()?,"sha256":timed(&mut profile.input_hash_us, || cache_identity::file_hash(path)).ok()?}),
    )
}
