//! Direct command cache contract observed through real subprocess launches.
#![cfg(unix)]
use super::{OrcaRunner, stages};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
#[path = "content_cache_tests/child.rs"]
mod child;

#[test]
fn direct_slice_content_cache_normalizes_paths_and_tracks_input_bytes() {
    verify("input", 2);
}
#[test]
fn direct_content_cache_unknown_arguments_bypass_before_hashing() {
    verify("unknown", 3);
}
#[test]
fn direct_content_cache_tracks_datadir_nested_bytes() {
    verify("datadir", 2);
}
#[test]
fn direct_content_cache_reuses_datadir_digest_and_invalidates_restored_mtime() {
    verify("memo", 2);
}
#[test]
fn direct_content_cache_tracks_datadir_names() {
    verify("rename", 2);
}
#[test]
fn direct_content_cache_tracks_empty_directories() {
    verify("directory", 2);
}
#[test]
fn direct_content_cache_separates_explicit_environment() {
    verify("explicit", 2);
}
#[test]
fn direct_content_cache_ignores_inherited_harness_environment() {
    verify("inherited", 1);
}
#[test]
fn direct_content_cache_corruption_falls_back_live() {
    verify("corrupt", 2);
}
#[test]
fn direct_content_cache_missing_capture_falls_back_live() {
    verify("missing", 2);
}
#[test]
fn direct_content_cache_wrong_capture_digest_falls_back_live() {
    verify("digest", 2);
}
#[test]
fn direct_content_cache_symlink_output_falls_back_live() {
    verify("symlink", 2);
}
#[test]
fn direct_content_cache_replays_rejection_without_changing_capture() {
    verify("rejection", 1);
}
#[test]
fn direct_content_cache_tampering_cannot_turn_native_failure_into_success() {
    verify("tampered-rejection", 2);
}
#[test]
fn direct_content_cache_never_publishes_timeouts() {
    verify("timeout", 3);
}
#[test]
fn direct_content_cache_never_publishes_crashes() {
    verify("crash", 3);
}
#[test]
fn direct_content_cache_header_alone_is_not_attributable_rejection() {
    verify("generic", 3);
}
#[test]
fn direct_content_cache_explicit_live_calls_bypass_lookup_and_publication() {
    verify("live", 3);
}
#[test]
fn direct_content_cache_export_file_outputs() {
    verify("export", 2);
}

fn corrupt(entry: &Path, change: &str, directory: &Path) {
    let root = entry.parent().unwrap();
    match change {
        "corrupt" => fs::write(entry, b"malformed").unwrap(),
        "missing" => fs::remove_file(root.join("stdout.bin")).unwrap(),
        "digest" => fs::write(root.join("stdout.bin"), b"wrong bytes").unwrap(),
        "tampered-rejection" => {
            let mut packet: serde_json::Value =
                serde_json::from_slice(&fs::read(entry).unwrap()).unwrap();
            packet["code"] = json!(0);
            packet["status"] = json!("exit status: 0");
            packet["rejection"] = serde_json::Value::Null;
            fs::write(entry, serde_json::to_vec(&packet).unwrap()).unwrap();
        }
        "symlink" => {
            let file = root.join("outputs/0/nested/result.bin");
            let target = directory.join("linked-output");
            fs::copy(&file, &target).unwrap();
            fs::remove_file(&file).unwrap();
            std::os::unix::fs::symlink(target, file).unwrap();
        }
        _ => unreachable!(),
    }
}

fn verify(change: &str, expected: usize) {
    let directory = tempfile::Builder::new()
        .prefix("direct-content-cache-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::create_dir(directory.join("resources")).unwrap();
    fs::write(directory.join("resources/value"), "resources").unwrap();
    fs::write(
        directory.join("wrapper.sh"),
        "#!/bin/sh\nexec \"$ARES_ORCA_NATIVE_BIN\" \"$@\"\n",
    )
    .unwrap();
    fs::write(
        directory.join("native.sh"),
        r#"#!/bin/sh
set -e
output= input= mode=0 export_path=
while [ "$#" -gt 0 ]; do
 case "$1" in
 --outputdir) output=$2; shift ;;
 --export-3mf) export_path=$2; shift ;;
 --slice) mode=$2; shift ;;
 --datadir) shift ;;
 esac
 input=$1
 shift
done
printf 'native\n' >> "$CONTENT_CACHE_ROOT/calls"
printf 'native stdout\n'
printf 'native stderr\n' >&2
case "$mode" in
 1) exec sleep 10 ;;
 2) kill -KILL $$ ;;
 3) printf 'Param values in 3mf/config error:\nlayer_height: invalid value\n' >&2; exit 238 ;;
 4) printf 'Param values in 3mf/config error: \n' >&2; exit 238 ;;
esac
if [ -n "$export_path" ]; then
 cat "$input" > "$export_path"
else
 mkdir -p "$output/nested"
 { cat "$input"; printf '%s' "$CONTENT_EXPLICIT"; } > "$output/nested/result.bin"
fi
"#,
    )
    .unwrap();
    for name in ["wrapper.sh", "native.sh"] {
        fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let steps = if matches!(change, "timeout" | "crash" | "live" | "generic") {
        4
    } else {
        3
    };
    for step in 1..=steps {
        let work = directory.join(step.to_string());
        fs::create_dir(&work).unwrap();
        let input = if matches!(change, "input" | "export") && step == 3 {
            "changed"
        } else {
            "original"
        };
        fs::write(work.join("input.3mf"), input).unwrap();
        fs::create_dir_all(work.join("data/nested")).unwrap();
        fs::write(
            work.join("data/nested/value"),
            if change == "datadir" && step == 3 {
                "changed"
            } else {
                "original"
            },
        )
        .unwrap();
        if change == "rename" && step == 3 {
            fs::rename(
                work.join("data/nested/value"),
                work.join("data/nested/renamed"),
            )
            .unwrap();
        }
        if change == "directory" && step == 3 {
            fs::create_dir(work.join("data/empty")).unwrap();
        }
        if matches!(
            change,
            "corrupt" | "missing" | "digest" | "symlink" | "tampered-rejection"
        ) && step == 3
        {
            let mut entries = Vec::new();
            receipts(&directory.join("cache"), &mut entries);
            assert!(
                !entries.is_empty(),
                "no command cache receipt: {directory:?}"
            );
            for entry in entries {
                corrupt(&entry, change, &directory);
            }
        }
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "runner::content_cache_tests::child::direct_content_cache_child",
                "--nocapture",
            ])
            .env("CONTENT_CACHE_ROOT", &directory)
            .env("CONTENT_CACHE_STEP", step.to_string())
            .env("CONTENT_CACHE_CHANGE", change)
            .env("ARES_PARITY_ARTIFACT_ROOT", directory.join("artifacts"))
            .env("ARES_ORCA_CACHE_DIR", directory.join("cache"))
            .env("ARES_ORCA_BIN", directory.join("wrapper.sh"))
            .env("ARES_ORCA_NATIVE_BIN", directory.join("native.sh"))
            .env("ARES_ORCA_RESOURCES_DIR", directory.join("resources"))
            .env(
                "ORCA_REPLAY_CPU",
                if change == "inherited" {
                    step.to_string()
                } else {
                    "1".into()
                },
            )
            .env("NEXTEST_CONTENT_CACHE_PROBE", step.to_string())
            .env_remove("CONTENT_EXPLICIT")
            .env_remove("ORCA_APPDIR")
            .env_remove("ORCA_REPLAY_BIN")
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("running 1 test"));
        assert!(output.status.success(), "{output:?}; {directory:?}");
    }
    let calls = fs::read_to_string(directory.join("calls")).unwrap();
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "change": change, "expected_launches": expected, "calls": calls,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(calls.lines().count(), expected, "evidence: {directory:?}");
}

fn receipts(directory: &Path, result: &mut Vec<std::path::PathBuf>) {
    if !directory.exists() {
        return;
    }
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            receipts(&path, result);
        } else if path.file_name().unwrap() == "result.json" {
            result.push(path);
        }
    }
}
