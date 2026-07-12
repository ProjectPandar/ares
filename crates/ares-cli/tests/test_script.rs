use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
#[test]
fn test_script_runs_workspace_with_parallel_nextest_by_default() {
    let args = run_test_script(&[]);

    assert_parallel_nextest_args(&args);
    assert_eq!(args, ["nextest", "run", "--workspace"]);
}

#[cfg(unix)]
#[test]
fn test_script_forwards_arguments_to_parallel_nextest() {
    let args = run_test_script(&["-p", "ares-core", "printable_height"]);

    assert_parallel_nextest_args(&args);
    assert_eq!(
        args,
        ["nextest", "run", "-p", "ares-core", "printable_height"]
    );
}

#[test]
fn test_script_executable_entrypoints_use_nextest_instead_of_cargo_test() {
    let entrypoints = executable_test_entrypoints();

    assert!(entrypoints.contains(&PathBuf::from("scripts/test.sh")));
    for path in entrypoints {
        let contents = fs::read_to_string(repo_root().join(&path)).unwrap();
        assert!(
            !contents.contains("cargo test"),
            "{} must use cargo nextest run so tests run through nextest parallel scheduling",
            path.display()
        );
        assert!(
            !contents.contains("RUST_TEST_THREADS"),
            "{} must not override test parallelism through RUST_TEST_THREADS",
            path.display()
        );
    }
}

#[test]
fn test_script_nextest_default_profile_runs_tests_in_parallel() {
    let config = fs::read_to_string(repo_root().join(".config/nextest.toml")).unwrap();
    let default_profile = nextest_profile(&config, "profile.default");

    assert!(default_profile.contains("default-filter = \"all()\""));
    assert!(default_profile.contains("test-threads = \"num-cpus\""));
    assert!(!default_profile.contains("test-threads = 1"));
    assert!(!default_profile.contains("test-threads = \"1\""));
    assert!(!default_profile.contains("test-threads = 0"));
}

#[test]
fn test_script_cargo_xtest_alias_uses_parallel_nextest() {
    let config = fs::read_to_string(repo_root().join(".cargo/config.toml")).unwrap();
    let alias_profile = nextest_profile(&config, "alias");

    assert!(
        alias_profile
            .lines()
            .any(|line| toml_alias_key(line) == Some("xtest") && line.contains("\"nextest run\"")),
        "cargo xtest must remain a nextest run alias"
    );
    assert!(
        !alias_profile
            .lines()
            .any(|line| toml_alias_key(line) == Some("test")),
        "cargo test alias must not shadow cargo xtest nextest entrypoint"
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn nextest_profile<'a>(config: &'a str, profile: &str) -> &'a str {
    let header = format!("[{profile}]");
    config
        .split(&header)
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap()
}

fn toml_alias_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let (key, _) = trimmed.split_once('=')?;
    let key = key.trim();
    Some(
        key.strip_prefix('"')
            .and_then(|key| key.strip_suffix('"'))
            .unwrap_or_else(|| {
                key.strip_prefix('\'')
                    .and_then(|key| key.strip_suffix('\''))
                    .unwrap_or(key)
            }),
    )
}

fn executable_test_entrypoints() -> Vec<PathBuf> {
    let root = repo_root();
    let mut paths = Vec::new();

    for dir in ["scripts", ".cargo", ".github"] {
        collect_files(&root, Path::new(dir), &mut paths);
    }
    for file in ["Makefile", "Justfile", "justfile"] {
        let path = PathBuf::from(file);
        if root.join(&path).is_file() {
            paths.push(path);
        }
    }

    paths.sort();
    paths
}

fn collect_files(root: &Path, relative_dir: &Path, paths: &mut Vec<PathBuf>) {
    let dir = root.join(relative_dir);
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries {
        let entry = entry.unwrap();
        let relative_path = relative_dir.join(entry.file_name());
        let path = root.join(&relative_path);
        if path.is_dir() {
            collect_files(root, &relative_path, paths);
        } else {
            paths.push(relative_path);
        }
    }
}

#[cfg(unix)]
fn assert_parallel_nextest_args(args: &[String]) {
    assert_eq!(&args[..2], ["nextest", "run"]);
    assert!(
        !args.iter().any(|arg| {
            arg == "--test-threads"
                || arg.starts_with("--test-threads=")
                || arg == "-j"
                || arg.starts_with("-j")
        }),
        "scripts/test.sh must leave nextest concurrency to .config/nextest.toml"
    );
}

#[cfg(unix)]
fn run_test_script(args: &[&str]) -> Vec<String> {
    let temp_dir = tempfile::tempdir().unwrap();
    let cargo_args_path = temp_dir.path().join("cargo-args");
    let cargo_path = temp_dir.path().join("cargo");
    fs::write(
        &cargo_path,
        "#!/usr/bin/env sh\nfor arg in \"$@\"; do\n  printf '%s\\n' \"$arg\"\ndone > \"$ARES_TEST_CARGO_ARGS\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&cargo_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&cargo_path, permissions).unwrap();

    let original_path = env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![temp_dir.path().to_path_buf()];
    paths.extend(env::split_paths(&original_path));
    let path = env::join_paths(paths).unwrap();

    let status = Command::new("bash")
        .arg(repo_root().join("scripts/test.sh"))
        .args(args)
        .env("PATH", path)
        .env("ARES_TEST_CARGO_ARGS", &cargo_args_path)
        .status()
        .unwrap();

    assert!(status.success());
    fs::read_to_string(cargo_args_path)
        .unwrap()
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}
