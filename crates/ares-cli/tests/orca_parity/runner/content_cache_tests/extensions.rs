//! Contract checks for stat-cached producer/datadir identity, CLI option flags, and repeat bypass.
#[path = "extensions_child.rs"]
mod child;

use super::super::OrcaRunner;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

fn write_fake_native(directory: &Path) {
    fs::create_dir_all(directory.join("resources")).unwrap();
    fs::write(directory.join("resources/value"), "resource-v1").unwrap();
    fs::write(directory.join("model.stl"), "model-v1").unwrap();
    fs::write(
        directory.join("wrapper.sh"),
        "#!/bin/sh\nexec \"$ARES_ORCA_NATIVE_BIN\" \"$@\"\n",
    )
    .unwrap();
    fs::write(
        directory.join("native.sh"),
        r#"#!/bin/sh
set -e
output= export_path= input=
while [ "$#" -gt 0 ]; do
 case "$1" in
  --outputdir) output=$2; shift ;;
  --export-3mf) export_path=$2; shift ;;
  --slice|--datadir|--load-settings|--load-filaments|--arrange|--debug|--layer-height|--post-process) shift ;;
  --*) ;;
  *) input=$1 ;;
 esac
 shift
done
printf 'native\n' >> "$EXT_CACHE_ROOT/calls"
if [ -n "$EXT_WRITE_CWD_RESULT" ]; then
 printf '{"return_code":0}\n' > result.json
fi
if [ -n "$export_path" ]; then
 printf 'exported-project' > "$export_path"
else
 mkdir -p "$output"
 cat "$input" > "$output/plate_1.gcode"
fi
"#,
    )
    .unwrap();
    for name in ["wrapper.sh", "native.sh"] {
        fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn find_traces(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.is_dir() {
        return;
    }
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            find_traces(&path, out);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("cache.json") {
            out.push(path);
        }
    }
}

fn single_trace(root: &Path) -> Value {
    let mut traces = Vec::new();
    find_traces(root, &mut traces);
    assert_eq!(traces.len(), 1);
    serde_json::from_slice(&fs::read(&traces[0]).unwrap()).unwrap()
}

fn slice_args(out: &Path, input: &Path) -> Vec<String> {
    vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        out.to_str().unwrap().into(),
        input.to_str().unwrap().into(),
    ]
}

fn run_child(directory: &Path, case: &str, step: &str, extra_env: &[(&str, &Path)]) {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "runner::content_cache_tests::extensions::cache_extension_child",
            "--nocapture",
        ])
        .env("EXT_CACHE_ROOT", directory)
        .env("EXT_CACHE_CASE", case)
        .env("EXT_CACHE_STEP", step)
        .env(
            "ARES_PARITY_ARTIFACT_ROOT",
            directory.join(format!("artifacts-{step}")),
        )
        .env("ARES_ORCA_CACHE_DIR", directory.join("cache"))
        .env("ARES_ORCA_BIN", directory.join("wrapper.sh"))
        .env("ARES_ORCA_NATIVE_BIN", directory.join("native.sh"))
        .env("ARES_ORCA_RESOURCES_DIR", directory.join("resources"))
        .env_remove("ORCA_APPDIR")
        .env_remove("ORCA_REPLAY_BIN")
        .env_remove("ARES_PARITY_PROJECT_PRESET_CATALOG")
        .env_remove("EXT_WRITE_CWD_RESULT");
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let output = command.output().unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
        "{output:?}"
    );
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn make_test_dir(prefix: &str) -> PathBuf {
    let root = crate::artifacts::root_from_env().unwrap_or_else(|_| std::env::temp_dir());
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(root)
        .unwrap()
        .keep();
    write_fake_native(&directory);
    directory
}

#[test]
fn orca_runner_producer_reuses_stat_cached_identity() {
    let directory = make_test_dir("cache-ext-producer-");
    run_child(&directory, "producer", "1", &[]);
    run_child(&directory, "producer", "2", &[]);
    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    assert_eq!(calls.lines().count(), 2);
    for (step, expected_hit) in [("1", false), ("2", true)] {
        let mut traces = Vec::new();
        find_traces(&directory.join(format!("artifacts-{step}")), &mut traces);
        assert!(!traces.is_empty());
        for trace in traces {
            let value: Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
            assert_eq!(value["producer"]["identity_cache_hit"], json!(expected_hit));
        }
    }
}

#[test]
fn immutable_supply_datadir_stat_caches_while_writable_datadir_hashes() {
    let directory = make_test_dir("cache-ext-datadir-");
    let supply = directory.join("supply");
    let native_dir = supply.join("native");
    fs::create_dir_all(native_dir.join("system")).unwrap();
    fs::write(native_dir.join("system/preset.json"), "{}").unwrap();
    let catalog = b"{}";
    let catalog_path = supply.join("catalog.json");
    fs::write(&catalog_path, catalog).unwrap();
    let digest: String = Sha256::digest(catalog)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    fs::write(
        supply.join("source-hashes.json"),
        serde_json::to_vec(&json!({"catalog_sha256": digest})).unwrap(),
    )
    .unwrap();
    fs::set_permissions(&native_dir, fs::Permissions::from_mode(0o555)).unwrap();

    for step in ["1", "2", "3"] {
        run_child(
            &directory,
            "datadir",
            step,
            &[("ARES_PARITY_PROJECT_PRESET_CATALOG", &catalog_path)],
        );
    }
    fs::set_permissions(&native_dir, fs::Permissions::from_mode(0o755)).unwrap();

    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    assert_eq!(calls.lines().count(), 2);
}

#[test]
fn cli_option_flags_cache_safe_scalars_and_bypass_bool_and_path_options() {
    let directory = make_test_dir("cache-ext-flags-");
    for step in [
        "scalar-1",
        "scalar-2",
        "bool-eq-1",
        "bool-eq-2",
        "bare-bool",
        "path-opt",
    ] {
        run_child(&directory, "flags", step, &[]);
    }
    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    assert_eq!(calls.lines().count(), 4);
}

#[test]
fn same_work_repeated_request_bypasses_cache_unconditionally() {
    let directory = make_test_dir("cache-ext-repeat-");
    run_child(&directory, "repeat", "1", &[]);
    run_child(
        &directory,
        "isolated-cwd",
        "2",
        &[("EXT_WRITE_CWD_RESULT", Path::new("1"))],
    );
    run_child(
        &directory,
        "isolated-cwd-hit",
        "3",
        &[("EXT_WRITE_CWD_RESULT", Path::new("1"))],
    );
    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    assert_eq!(calls.lines().count(), 4);
}

#[test]
fn cross_workdir_repeated_request_bypasses_cache_on_same_thread() {
    let directory = make_test_dir("cache-ext-cross-repeat-");
    run_child(&directory, "cross-repeat", "1", &[]);
    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    assert_eq!(calls.lines().count(), 2);
}

#[test]
fn cache_extension_child() {
    let Some(root) = std::env::var_os("EXT_CACHE_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let case = std::env::var("EXT_CACHE_CASE").unwrap();
    let step = std::env::var("EXT_CACHE_STEP").unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    child::run(&case, &root, &runner, &step);
}
