//! GCode.cpp:2157,9395,9559: selected-nozzle inner-perimeter speed.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    io::{Cursor, Write},
};

#[test]
fn native_inner_wall_original_index_nil_override() {
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    let mapped = super::mapped_nozzle::mapped_project(&source, 2);
    let configured = super::input::settings_variant(
        &mapped,
        json!({
            "inner_wall_speed":["19","23","37","53"],
            "small_perimeter_threshold":"0", "small_perimeter_speed":"100%",
            "enable_overhang_speed":["0","0","0","0"]
        })
        .as_object()
        .unwrap(),
    );
    let mut archive = zip::ZipArchive::new(Cursor::new(&configured)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut bytes = Vec::new();
            std::io::copy(&mut entry, &mut bytes).unwrap();
            let xml = std::str::from_utf8(&bytes).unwrap();
            let marker = "<object id=\"2\">";
            assert_eq!(xml.matches(marker).count(), 1);
            let updated = xml.replace(
                marker,
                &format!(
                    "{marker}\n    <metadata key=\"inner_wall_speed\" value=\"41,43,nil,47\"/>"
                ),
            );
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(updated.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    let project = writer.finish().unwrap().into_inner();
    // Original index2 selects nil, inheriting37. A reduced-index read selects43.
    let (reference, _) = native_stream_case::qualify(
        (
            "inner-wall-original-index-nil-override",
            &[
                "PrintObject.cpp:4363-4374 apply_to_print_region_config",
                "PrintConfig.cpp:11421-11429 set_variant_override",
                "Config.hpp:612-629 set_to_index nil inherits existing target",
            ],
        ),
        "physical-2",
        &project,
        &sha(&project),
        false,
    );
    let native = inner_feedrates(&reference.native);
    assert!(native.contains("2220"), "Native inherited37: {native:?}");
    for wrong in ["2460", "2580", "2820"] {
        assert!(!native.contains(wrong), "Native wrong override: {native:?}");
    }
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ));
    let api = result.as_ref().ok().map(|bytes| {
        fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
        inner_feedrates(std::str::from_utf8(bytes).unwrap())
    });
    fs::write(
        reference.root.join("original-index-nil-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":sha(&project),"process":[19,23,37,53],
            "object_override":[41,43,null,47],"physical":2,"original_index":2,
            "expected_inherited_speed":37,"native":native,"api":api,
            "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
            "matches":api.as_ref()==Some(&native),"whole_output_parity_claimed":false
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        api.as_ref(),
        Some(&native),
        "override: {:?}",
        reference.root
    );
}

fn inner_feedrates(text: &str) -> BTreeSet<String> {
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
        if role == "Inner wall"
            && line.starts_with("G1 ")
            && line.contains(" E")
            && (line.contains(" X") || line.contains(" Y"))
        {
            result.insert(feed.to_owned());
        }
    }
    result
}

#[test]
fn native_inner_wall_selected_nozzle_feedrate() {
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    assert_eq!(
        sha(&source),
        "6d576ce9d4835dfae9ec5edacf9c215d7d6893ce097606f06f987db98261ff34"
    );
    let mut failures = Vec::new();
    for physical in [1u8, 2] {
        let mapped = super::mapped_nozzle::mapped_project(&source, physical);
        let project = super::input::settings_variant(
            &mapped,
            json!({
                "inner_wall_speed":["19","23","37","53"],
                "small_perimeter_threshold":"0", "small_perimeter_speed":"100%",
                "enable_overhang_speed":["0","0","0","0"]
            })
            .as_object()
            .unwrap(),
        );
        let (reference, _) = native_stream_case::qualify(
            (
                "inner-wall-selected-nozzle",
                &["GCode.cpp:2157,9395,9559;PrintConfig.cpp:5957-5966"],
            ),
            &format!("physical-{physical}"),
            &project,
            &sha(&project),
            false,
        );
        let native = inner_feedrates(&reference.native);
        let expected = if physical == 1 { "1140" } else { "2220" };
        assert!(
            native.contains(expected),
            "Native selected-nozzle premise: {native:?}; {:?}",
            reference.root
        );
        assert!(!native.contains(if physical == 1 { "2220" } else { "1140" }));
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(crate::runner::application::slice_project_plate_validated(
                &project,
                GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                None,
                &ConfigOverrides::default(),
                SlicerContext::new(0),
            ));
        let api = result.as_ref().ok().map(|bytes| {
            fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
            inner_feedrates(std::str::from_utf8(bytes).unwrap())
        });
        let matches = api.as_ref() == Some(&native);
        fs::write(
            reference.root.join("inner-reader-comparison.json"),
            serde_json::to_vec_pretty(&json!({
                "physical":physical,"input_sha256":sha(&project),"native":native,"api":api,
                "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
                "selected_reader_matches":matches,"whole_output_parity_claimed":false
            }))
            .unwrap(),
        )
        .unwrap();
        if !matches {
            failures.push(reference.root);
        }
    }
    assert!(
        failures.is_empty(),
        "selected reader mismatch: {failures:?}"
    );
}
