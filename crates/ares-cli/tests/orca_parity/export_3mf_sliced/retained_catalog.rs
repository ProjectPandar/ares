use std::{fs, os::unix::fs::symlink, path::Path};

use serde_json::{Value, json};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{GCODE, compare_sliced_archives, invoke, write_saved_input};
use crate::{OrcaRunner, runner::repo_root};

const EXPANSION: &str = "raft_first_layer_expansion";
const ARCHIVE: &str = "export.3mf";
const INPUT: &str = "saved.3mf";

#[test]
fn direct_sliced_export_refreshes_unlisted_raft_expansion_from_retained_process() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let (native, ares) = run_pair(&runner, "wall_loops");
    assert_eq!(native, Some(0), "Native export in {root:?}");
    let native_archive = root.join("native/cwd").join(ARCHIVE);
    let ares_archive = root.join("ares/cwd").join(ARCHIVE);
    assert_effective_values(&native_archive);
    assert_eq!(ares, native, "Ares export in {root:?}");
    assert_effective_values(&ares_archive);
    compare_sliced_archives(&native_archive, &ares_archive);
}

#[test]
fn direct_sliced_export_rejects_authored_negative_raft_expansion_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let (native, ares) = run_pair(&runner, "wall_loops;raft_first_layer_expansion");
    assert_eq!(native, Some(238), "Native strict validation in {root:?}");
    for side in ["native", "ares"] {
        let dir = root.join(side);
        assert!(!dir.join("cwd").join(ARCHIVE).exists(), "{dir:?}");
        let stderr = fs::read_to_string(dir.join("stderr.log")).unwrap();
        assert!(
            stderr.contains(&format!("{EXPANSION}: -1 not in range")),
            "{side} must reject the authored value in {root:?}"
        );
    }
    assert_eq!(ares, native, "strict validation exit status in {root:?}");
}

fn run_pair(runner: &OrcaRunner, authored: &str) -> (Option<i32>, Option<i32>) {
    let root = runner.work_dir();
    let source = root.join("source/cwd");
    fs::create_dir_all(&source).unwrap();
    write_saved_input(runner, &source);
    let process: Value =
        serde_json::from_slice(&fs::read(source.join("process.json")).unwrap()).unwrap();
    let process_name = process["name"].as_str().unwrap();
    let saved = fs::read(source.join(INPUT)).unwrap();
    let project = rewrite_project(&saved, |name, text| {
        if name != "Metadata/project_settings.config" {
            return text.to_owned();
        }
        let mut config: Value = serde_json::from_str(text).unwrap();
        config[EXPANSION] = json!("-1");
        config["wall_loops"] = json!("3");
        config["different_settings_to_system"] = json!([authored, "", ""]);
        config["inherits_group"] = json!([process_name, "", ""]);
        serde_json::to_string(&config).unwrap()
    });

    let install = root.join("install");
    fs::create_dir_all(install.join("bin")).unwrap();
    symlink(
        repo_root().join("OrcaSlicer/resources"),
        install.join("resources"),
    )
    .unwrap();
    let installed = install.join("bin/ares");
    fs::copy(env!("CARGO_BIN_EXE_ares"), &installed).unwrap();
    let argv = ["--slice", "1", "--export-3mf", ARCHIVE, INPUT];
    let run = |side: &str, bin: &Path| {
        let cwd = root.join(side).join("cwd");
        fs::create_dir_all(cwd.join("out")).unwrap();
        fs::write(cwd.join(INPUT), &project).unwrap();
        invoke(bin, &cwd, &argv)
    };
    (run("native", runner.bin()), run("ares", &installed))
}

fn assert_effective_values(archive: &Path) {
    let members = super::read_members(archive);
    let plate = members.iter().find(|entry| entry.name == GCODE).unwrap();
    let gcode = std::str::from_utf8(&plate.bytes).unwrap();
    for expected in ["; raft_first_layer_expansion = 2", "; wall_loops = 3"] {
        assert!(
            gcode.lines().any(|line| line == expected),
            "missing {expected:?} in {archive:?}"
        );
    }
}
