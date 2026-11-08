//! PrintObject.cpp:4363–4374; Fill.cpp:1081; GCode.cpp:9559: selected process slots.
use super::super::super::{
    OrcaRunner, application,
    native_api_cli::{NativeCase, sha},
    repo_root,
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    io::{Cursor, Write},
};

#[test]
fn native_process_sparse_speed_selected_nozzle() {
    check(false);
}

#[test]
fn native_process_sparse_speed_original_index_nil_override() {
    check(true);
}

fn check(with_override: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    let mut failures = Vec::new();
    let mut native_slots = Vec::new();
    for physical in [1, 2] {
        let mapped = super::mapped_nozzle::mapped_project(&source, physical);
        let mut project = super::input::settings_variant(
            &mapped,
            json!({
                "sparse_infill_speed":["19","23","37","53"],
                "enable_overhang_speed":["0","0","0","0"],
                "small_perimeter_threshold":"0", "small_perimeter_speed":"100%"
            })
            .as_object()
            .unwrap(),
        );
        if with_override {
            project = object_override(&project);
        }
        let label = format!("process-sparse-reader-{physical}-{with_override}-");
        let reference = NativeCase::slice(&runner, &project, &label);
        let replay = NativeCase::slice(&runner, &project, &format!("{label}replay-"));
        let native = feedrates(&reference.native);
        assert!(!native.is_empty(), "Native must emit sparse infill");
        assert_eq!(native, feedrates(&replay.native));
        native_slots.push(native.clone());
        let api = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(application::slice_project_plate_validated(
                &project,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                None,
                &ares_core::ConfigOverrides::default(),
                ares_core::SlicerContext::new(0),
            ));
        let api_feeds = api.as_ref().ok().map(|bytes| {
            fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
            feedrates(std::str::from_utf8(bytes).unwrap())
        });
        let cli = application::cli_command()
            .args(["slice", "-o"])
            .arg(reference.root.join("ares.cli.gcode"))
            .arg(reference.root.join("input.3mf"))
            .output()
            .unwrap();
        fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
        fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
        let cli_bytes = fs::read(reference.root.join("ares.cli.gcode"));
        let cli_feeds = cli_bytes
            .as_ref()
            .ok()
            .map(|bytes| feedrates(std::str::from_utf8(bytes).unwrap()));
        let matches = api_feeds.as_ref() == Some(&native)
            && cli_feeds.as_ref() == Some(&native)
            && cli.status.success();
        fs::write(reference.root.join("process-reader.json"), serde_json::to_vec_pretty(&json!({
            "physical":physical, "with_override":with_override, "input_sha256":sha(&project),
            "native_replay_root":replay.root, "native":native, "api":api_feeds, "cli":cli_feeds,
            "api_error":api.as_ref().err().map(|error|format!("{error:?}")), "cli_code":cli.status.code(),
            "complete_api_comparison":api.as_ref().map(|bytes|crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), bytes)).map_err(|error|format!("{error:?}")),
            "matches":matches, "whole_output_parity_claimed":false,
        })).unwrap()).unwrap();
        if !matches {
            failures.push(reference.root);
        }
    }
    assert_ne!(
        native_slots[0], native_slots[1],
        "Native must distinguish the selected slots"
    );
    assert!(failures.is_empty(), "selected sparse speed: {failures:?}");
}

fn object_override(project: &[u8]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut bytes = Vec::new();
            std::io::copy(&mut entry, &mut bytes).unwrap();
            let xml = std::str::from_utf8(&bytes).unwrap();
            let marker = "<object id=\"2\">";
            assert_eq!(xml.matches(marker).count(), 1);
            let xml = xml.replace(
                marker,
                &format!(
                    "{marker}\n<metadata key=\"sparse_infill_speed\" value=\"41,43,nil,47\"/>"
                ),
            );
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(xml.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

fn feedrates(text: &str) -> BTreeSet<String> {
    let mut role = "";
    let mut feed = "";
    let mut result = BTreeSet::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("; FEATURE: ") {
            role = value;
        }
        for word in line.split_ascii_whitespace() {
            if let Some(value) = word.strip_prefix('F') {
                feed = value;
            }
        }
        if role == "Sparse infill"
            && line.starts_with("G1 ")
            && line.contains(" E")
            && (line.contains(" X") || line.contains(" Y"))
        {
            result.insert(feed.to_owned());
        }
    }
    result
}
