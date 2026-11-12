//! Native CLI export replay must retain its implicit cwd artifacts.
#![cfg(target_os = "linux")]
use super::super::{OrcaRunner, repo_root, stages, write_preset};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path, process::Command};

fn traces(root: &Path) -> BTreeSet<std::path::PathBuf> {
    fs::read_dir(root)
        .unwrap()
        .flat_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_dir() {
                traces(&path)
            } else if path.file_name().unwrap() == "cache.json" {
                BTreeSet::from([path])
            } else {
                BTreeSet::new()
            }
        })
        .collect()
}

#[test]
fn export_without_outputdir_replays_native_cwd_outputs() {
    if crate::native_environment::binary("ARES_ORCA_BIN").is_none() {
        return;
    }
    let root = tempfile::Builder::new()
        .prefix("native-cwd-cache-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::create_dir(root.join("native")).unwrap();
    fs::create_dir(root.join("reference")).unwrap();
    let profiles = crate::presets::VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "iQ",
    )
    .unwrap();
    let machine = profiles.machine("iQ TiQ2 0.25 Nozzle").unwrap();
    let process = machine["default_print_profile"].as_str().unwrap();
    let filament = machine["default_filament_profile"][0].as_str().unwrap();
    write_preset(&root.join("machine.json"), "machine", &machine).unwrap();
    write_preset(
        &root.join("process.json"),
        "process",
        &profiles.process(process).unwrap(),
    )
    .unwrap();
    write_preset(
        &root.join("filament.json"),
        "filament",
        &profiles.filament(filament).unwrap(),
    )
    .unwrap();
    for step in 1..=2 {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "runner::content_cache_tests::cwd_outputs::native_export_cwd_child",
                "--nocapture",
            ])
            .env("NATIVE_CWD_CACHE_ROOT", &root)
            .env("NATIVE_CWD_CACHE_STEP", step.to_string())
            .env("ARES_ORCA_CACHE_DIR", root.join("cache"))
            .env("ARES_PARITY_ARTIFACT_ROOT", root.join("artifacts"))
            .output()
            .unwrap();
        fs::write(root.join(format!("child-{step}.stdout")), &output.stdout).unwrap();
        fs::write(root.join(format!("child-{step}.stderr")), &output.stderr).unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("running 1 test"));
        assert!(output.status.success(), "{output:?}; artifacts {root:?}");
        if step == 1 {
            let work = root.join("native");
            for name in ["result.json", "00000.log"] {
                fs::copy(work.join(name), root.join("reference").join(name)).unwrap();
                fs::remove_file(work.join(name)).unwrap();
            }
            fs::remove_file(root.join("export.3mf")).unwrap();
        }
    }
}

#[test]
fn native_export_cwd_child() {
    let Some(root) = std::env::var_os("NATIVE_CWD_CACHE_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let step = std::env::var("NATIVE_CWD_CACHE_STEP").unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let work = root.join("native");
    let before = traces(&work);
    stages::run(
        runner.bin(),
        [
            "--load-settings".into(),
            format!(
                "{};{}",
                root.join("machine.json").display(),
                root.join("process.json").display()
            ),
            "--load-filaments".into(),
            root.join("filament.json").to_str().unwrap().into(),
            "--arrange".into(),
            "1".into(),
            "--export-3mf".into(),
            root.join("export.3mf").to_str().unwrap().into(),
            repo_root()
                .join("tests/parity/cube10.stl")
                .to_str()
                .unwrap()
                .into(),
        ],
        stages::Stage::Export,
        &work,
    )
    .unwrap();
    let after = traces(&work);
    let added = after.difference(&before).collect::<Vec<_>>();
    assert_eq!(added.len(), 1);
    let trace: Value = serde_json::from_slice(&fs::read(added[0]).unwrap()).unwrap();
    assert_eq!(trace["outcome"], if step == "1" { "miss" } else { "hit" });
    let result = fs::read(work.join("result.json")).unwrap();
    let log = fs::read(work.join("00000.log")).unwrap();
    let parsed: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(parsed["return_code"], 0);
    if step == "2" {
        assert_eq!(
            result,
            fs::read(root.join("reference/result.json")).unwrap()
        );
        assert_eq!(log, fs::read(root.join("reference/00000.log")).unwrap());
    }
    fs::write(
        root.join(format!("step-{step}.json")),
        serde_json::to_vec_pretty(&json!({
            "trace": trace, "result": parsed, "result_bytes": result.len(), "log_bytes": log.len(),
            "result_sha256": super::super::cache_identity::raw_digest(&result),
            "log_sha256": super::super::cache_identity::raw_digest(&log),
        }))
        .unwrap(),
    )
    .unwrap();
}
