#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path, process::Command};

use crate::{artifacts, runner::OrcaRunner};

enum Destination<'a> {
    Default,
    Empty,
    Relative(&'a str),
    ExistingAbsolute(&'a str),
}

#[test]
fn cli_logfile_relative_to_cwd_at_default_level() {
    compare(
        None,
        Destination::Relative("diagnostics.log"),
        197,
        &["diagnostics.log", "out/", "out/result.json"],
    );
}

#[test]
fn cli_logfile_replaces_existing_absolute_file() {
    compare(
        Some("1"),
        Destination::ExistingAbsolute("out/diagnostics.log"),
        197,
        &["out/", "out/diagnostics.log", "out/result.json"],
    );
}

#[test]
fn cli_logfile_creates_missing_parents_when_logging_error() {
    compare(
        Some("1"),
        Destination::Relative("logs/detail/diagnostics.log"),
        197,
        &[
            "logs/",
            "logs/detail/",
            "logs/detail/diagnostics.log",
            "out/",
            "out/result.json",
        ],
    );
}

#[test]
fn cli_logfile_empty_path_uses_default_file() {
    compare(
        Some("1"),
        Destination::Empty,
        197,
        &["00000.log", "out/", "out/result.json"],
    );
}

#[test]
fn cli_debug_zero_creates_no_default_log() {
    compare(
        Some("0"),
        Destination::Default,
        197,
        &["out/", "out/result.json"],
    );
}

#[test]
fn cli_debug_zero_does_not_create_logfile_parents() {
    compare(
        Some("0"),
        Destination::Relative("logs/detail/diagnostics.log"),
        197,
        &["out/", "out/result.json"],
    );
}

#[test]
fn cli_logfile_is_not_created_without_a_diagnostic() {
    compare(
        Some("1"),
        Destination::Relative("diagnostics.log"),
        0,
        &["out/", "out/settings.json", "result.json"],
    );
}

fn compare(debug: Option<&str>, destination: Destination<'_>, exit: i32, expected: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-debug-logfile-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    super::cli_mtcpp::write_fixture(&root, true);

    let mut argv: Vec<OsString> = Vec::new();
    if let Some(debug) = debug {
        argv.extend(["--debug".into(), debug.into()]);
    }
    let log = match destination {
        Destination::Default => "00000.log",
        Destination::Empty => {
            argv.extend(["--logfile".into(), "".into()]);
            "00000.log"
        }
        Destination::Relative(path) => {
            argv.extend(["--logfile".into(), path.into()]);
            path
        }
        Destination::ExistingAbsolute(path) => {
            argv.extend(["--logfile".into(), root.join("cwd").join(path).into()]);
            path
        }
    };
    let action: &[&str] = if exit == 0 {
        &["--export-settings", "out/settings.json"]
    } else {
        &["--mtcpp", "11", "--slice", "1", "--outputdir", "out"]
    };
    argv.extend(action.iter().copied().map(OsString::from));
    argv.extend([
        "--load-settings".into(),
        format!(
            "{};{}",
            root.join("machine.json").display(),
            root.join("process.json").display()
        )
        .into(),
        "--load-filaments".into(),
        root.join("filament.json").into(),
        root.join("first.stl").into(),
    ]);

    let existing = matches!(destination, Destination::ExistingAbsolute(_)).then_some(log);
    let native = run(runner.bin(), &root, "native", &argv, existing);
    let ares = run(
        Path::new(env!("CARGO_BIN_EXE_ares")),
        &root,
        "ares",
        &argv,
        existing,
    );
    assert_eq!(native.status.code(), Some(exit), "{root:?}");
    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    assert_eq!(inventory(&native_cwd), expected, "{root:?}");
    let native_bytes = expected
        .contains(&log)
        .then(|| fs::read(native_cwd.join(log)).unwrap());
    if let Some(bytes) = &native_bytes {
        assert!(
            bytes == b"plate 1: triangle count 12 exceeds the limit:11\n",
            "unexpected Native log bytes: {root:?}"
        );
    }

    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_cwd), expected, "{root:?}");
    if let Some(bytes) = native_bytes {
        assert!(
            fs::read(ares_cwd.join(log)).unwrap() == bytes,
            "log bytes differ: {root:?}"
        );
    }
}

fn run(
    bin: &Path,
    root: &Path,
    engine: &str,
    argv: &[OsString],
    existing: Option<&str>,
) -> std::process::Output {
    let work = root.join(engine);
    let cwd = root.join("cwd");
    let tmp = work.join("tmp");
    fs::create_dir_all(cwd.join("out")).unwrap();
    fs::create_dir_all(&tmp).unwrap();
    if let Some(path) = existing {
        fs::write(cwd.join(path), b"previous invocation\n").unwrap();
    }
    let result = Command::new(bin)
        .current_dir(&cwd)
        .env("TMPDIR", tmp)
        .env("ORCA_250_JOB_APPDIR", work.join("app"))
        .env("ORCA_JOB_APPDIR", work.join("app"))
        .args(argv)
        .output()
        .unwrap();
    fs::rename(cwd, work.join("cwd")).unwrap();
    fs::write(work.join("exit"), format!("{:?}\n", result.status.code())).unwrap();
    fs::write(work.join("stderr"), &result.stderr).unwrap();
    fs::write(work.join("stdout"), &result.stdout).unwrap();
    result
}

fn inventory(cwd: &Path) -> Vec<String> {
    fn visit(cwd: &Path, dir: &Path, paths: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(cwd).unwrap().to_str().unwrap();
            if entry.file_type().unwrap().is_dir() {
                paths.push(format!("{relative}/"));
                visit(cwd, &path, paths);
            } else {
                paths.push(relative.to_owned());
            }
        }
    }
    let mut paths = Vec::new();
    visit(cwd, cwd, &mut paths);
    paths.sort_unstable();
    paths
}
