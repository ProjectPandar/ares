//! Cache contract through export/slice subprocesses, not hash internals.
#![cfg(unix)]
use super::{CaseInputs, OrcaRunner};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn native_cache_identical_content_hits_across_runners() {
    verify("same", 2);
}
#[test]
fn native_cache_changed_actual_image_misses() {
    verify("image", 4);
}
#[test]
fn native_cache_changed_resource_file_misses() {
    verify("resources", 4);
}
#[test]
fn native_cache_changed_actual_3mf_misses() {
    verify("project", 3);
}
#[test]
fn native_cache_changed_seed_misses() {
    verify("seed", 4);
}

fn verify(change: &str, expected_calls: usize) {
    let root = crate::artifacts::root_from_env().unwrap_or_else(|_| std::env::temp_dir());
    let directory = tempfile::Builder::new()
        .prefix("native-cache-contract-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    fs::create_dir(directory.join("resources")).unwrap();
    fs::write(directory.join("resources/value"), "resource1").unwrap();
    fs::write(directory.join("model.stl"), "model").unwrap();
    fs::write(
        directory.join("wrapper.sh"),
        "#!/bin/sh\nexec \"$ARES_ORCA_NATIVE_BIN\" \"$@\"\n",
    )
    .unwrap();
    fs::write(
        directory.join("native.sh"),
        r#"#!/bin/sh
set -e
export_path= output= input=
while [ "$#" -gt 0 ]; do
 case "$1" in
 --export-3mf) export_path=$2; shift ;;
 --outputdir) output=$2; shift ;;
 esac
 input=$1
 shift
done
if [ -n "$export_path" ]; then
 printf 'export\n' >> "$CACHE_CONTRACT_ROOT/calls"
 printf 'original-project' > "$export_path"
else
 printf 'slice\n' >> "$CACHE_CONTRACT_ROOT/calls"
 { sha256sum "$ARES_ORCA_NATIVE_BIN" | cut -d ' ' -f1
   cat "$ARES_ORCA_RESOURCES_DIR/value"
   printf '\n%s\n' "$ORCA_REPLAY_SEED"
   sha256sum "$input" | cut -d ' ' -f1
 } > "$output/plate_1.gcode"
fi
"#,
    )
    .unwrap();
    for name in ["wrapper.sh", "native.sh"] {
        fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o755)).unwrap();
    }
    invoke(&directory, change, "1");
    match change {
        "image" => {
            let path = directory.join("native.sh");
            let mut bytes = fs::read(&path).unwrap();
            bytes.extend_from_slice(b"\n# image2\n");
            fs::write(path, bytes).unwrap();
        }
        "resources" => fs::write(directory.join("resources/value"), "resource2").unwrap(),
        "same" | "project" | "seed" => (),
        _ => unreachable!(),
    }
    invoke(&directory, change, "2");
    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    let first = fs::read(directory.join("reference-1")).unwrap();
    let second = fs::read(directory.join("reference-2")).unwrap();
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "change": change, "expected_native_calls": expected_calls, "calls":calls,
            "outputs_equal":first==second,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        calls.lines().count(),
        expected_calls,
        "{change}: {directory:?}"
    );
    assert_eq!(first == second, change == "same", "{change}: {directory:?}");
}

fn invoke(directory: &std::path::Path, change: &str, step: &str) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runner::cache_contract_tests::native_cache_contract_child",
            "--nocapture",
        ])
        .env("CACHE_CONTRACT_ROOT", directory)
        .env("CACHE_CONTRACT_CHANGE", change)
        .env("CACHE_CONTRACT_STEP", step)
        .env("ARES_PARITY_ARTIFACT_ROOT", directory.join("artifacts"))
        .env("ARES_ORCA_CACHE_DIR", directory.join("cache"))
        .env("ARES_ORCA_BIN", directory.join("wrapper.sh"))
        .env("ARES_ORCA_NATIVE_BIN", directory.join("native.sh"))
        .env("ARES_ORCA_RESOURCES_DIR", directory.join("resources"))
        .env(
            "ORCA_REPLAY_SEED",
            if change == "seed" && step == "2" {
                "2"
            } else {
                "1"
            },
        )
        .env_remove("ORCA_APPDIR")
        .env_remove("ORCA_REPLAY_BIN")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn native_cache_contract_child() {
    let Some(directory) = std::env::var_os("CACHE_CONTRACT_ROOT") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let step = std::env::var("CACHE_CONTRACT_STEP").unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let machine = serde_json::from_value(json!({"name":"machine"})).unwrap();
    let process = serde_json::from_value(json!({"name":"process"})).unwrap();
    let inputs = CaseInputs {
        label: "cache-contract",
        machine: &machine,
        process: &process,
        filaments: &[],
    };
    let mut exported = runner
        .export_case(
            &inputs,
            &serde_json::Map::new(),
            &directory.join("model.stl"),
        )
        .unwrap();
    if step == "2" && std::env::var("CACHE_CONTRACT_CHANGE").unwrap() == "project" {
        exported.project = b"changed-project".to_vec();
        fs::write(&exported.project_path, &exported.project).unwrap();
    }
    let actual = runner.slice_case(&exported).unwrap();
    fs::write(
        directory.join(format!("reference-{step}")),
        actual.reference,
    )
    .unwrap();
}
