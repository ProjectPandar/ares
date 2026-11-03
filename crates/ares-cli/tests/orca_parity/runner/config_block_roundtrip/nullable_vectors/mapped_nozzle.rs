use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    io::{Cursor, Write},
};

pub(super) fn mapped_project(source: &[u8], physical: u8) -> Vec<u8> {
    let updates = json!({
        "filament_map_mode": "Manual",
        "filament_map": [physical.to_string(), physical.to_string()],
        "enable_prime_tower": "0", "slow_down_for_layer_cooling": ["0", "0"],
        "outer_wall_speed": ["17", "21", "33", "47"],
        "enable_overhang_speed": ["0", "0", "1", "1"],
        "slowdown_for_curled_perimeters": ["0", "0", "1", "1"],
    });
    let input = super::input::settings_variant(source, updates.as_object().unwrap());
    let mut archive = zip::ZipArchive::new(Cursor::new(&input)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut bytes = Vec::new();
            std::io::copy(&mut entry, &mut bytes).unwrap();
            let original = std::str::from_utf8(&bytes).unwrap();
            let old = "<metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>";
            assert_eq!(original.matches(old).count(), 1);
            let updated = original.replace(
                old,
                "<metadata key=\"filament_map_mode\" value=\"Manual\"/>",
            );
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(updated.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

fn config_token<'a>(text: &'a str, key: &str) -> &'a str {
    let prefix = format!("; {key} = ");
    text.lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .next_back()
        .unwrap()
}

// Observe one output channel, not a replacement whole-G-code comparator.
// First-layer outer walls retain their independent initial-layer feedrate.
pub(super) fn outer_feedrates(text: &str) -> BTreeSet<String> {
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
        if role == "Outer wall"
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
fn native_nonzero_nozzle_slot_selects_public_outer_wall_feedrate() {
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    assert_eq!(
        sha(&source),
        "6d576ce9d4835dfae9ec5edacf9c215d7d6893ce097606f06f987db98261ff34"
    );
    let mut failures = Vec::new();
    for physical in [1u8, 2] {
        let project = mapped_project(&source, physical);
        let (reference, _) = native_stream_case::qualify(
            (
                "nullable-nozzle-slot",
                &[
                    "PrintApply.cpp:1429 process variant expansion (345efec64c7b)",
                    "GCode.cpp:2157,5055-5061,9400 NOZZLE_CONFIG current filament",
                    "Print.cpp:5107-5180 mapped nozzle config index",
                ],
            ),
            &format!("physical-{physical}"),
            &project,
            &sha(&project),
            false,
        );
        assert_eq!(config_token(&reference.native, "outer_wall_speed"), "17,33");
        assert_eq!(config_token(&reference.native, "print_extruder_id"), "1,2");
        assert_eq!(
            config_token(&reference.native, "enable_overhang_speed"),
            "0,1"
        );
        assert_eq!(
            config_token(&reference.native, "slowdown_for_curled_perimeters"),
            "0,1"
        );
        let expected_map = if physical == 1 { "0,0" } else { "1,1" };
        assert_eq!(
            config_token(&reference.native, "filament_map_2"),
            expected_map
        );
        let expected = if physical == 1 { "1020" } else { "1980" };
        let native_feedrates = outer_feedrates(&reference.native);
        assert!(
            native_feedrates.contains(expected),
            "Native must select slot: {:?}",
            reference.root
        );
        assert!(!native_feedrates.contains(if physical == 1 { "1980" } else { "1020" }));
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
        let mut api_feedrates = None;
        if let Ok(api) = &result {
            fs::write(reference.root.join("ares.api.gcode"), api).unwrap();
            api_feedrates = Some(outer_feedrates(std::str::from_utf8(api).unwrap()));
        }
        let matches = api_feedrates.as_ref() == Some(&native_feedrates);
        fs::write(
            reference.root.join("mapped-nozzle-comparison.json"),
            serde_json::to_vec_pretty(&json!({
                "physical_extruder": physical, "applied_config_slot": physical - 1,
                "source_sha256": sha(&source), "input_sha256": sha(&project),
                "native_feedrates": native_feedrates, "api_feedrates": api_feedrates,
                "api_error": result.as_ref().err().map(|error| format!("{error:?}")),
                "feedrate_channel_matches": matches, "complete_gcode_parity_claimed": false,
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
        "mapped nozzle public output: {failures:?}"
    );
}

#[test]
fn native_outer_wall_volumetric_placeholder_selects_physical_nozzle() {
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    let mut failures = Vec::new();
    let mut native_values = Vec::new();
    for physical in [1u8, 2] {
        let mapped = mapped_project(&source, physical);
        let project = super::input::settings_variant(
            &mapped,
            json!({"machine_start_gcode": "; NULLABLE_OUTER_VOLUME {outer_wall_volumetric_speed}\n"})
                .as_object().unwrap(),
        );
        let (reference, _) = native_stream_case::qualify(
            (
                "nullable-nozzle-placeholder",
                &[
                    "GCode.cpp:760-774 get_outer_wall_volumetric_speed physical extruder_id",
                    "GCode.cpp:4034-4037 initial non-support filament placeholder",
                ],
            ),
            &format!("physical-{physical}"),
            &project,
            &sha(&project),
            false,
        );
        let channel = |text: &str| {
            text.lines()
                .find_map(|line| line.strip_prefix("; NULLABLE_OUTER_VOLUME "))
                .unwrap()
                .to_owned()
        };
        let native = channel(&reference.native);
        native_values.push(native.clone());
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
            channel(std::str::from_utf8(bytes).unwrap())
        });
        let matches = api.as_ref() == Some(&native);
        fs::write(
            reference.root.join("mapped-placeholder-comparison.json"),
            serde_json::to_vec_pretty(&json!({
                "physical_extruder": physical, "input_sha256": sha(&project),
                "native": native, "api": api, "channel_matches": matches,
                "api_error": result.as_ref().err().map(|error| format!("{error:?}")),
                "complete_gcode_parity_claimed": false,
            }))
            .unwrap(),
        )
        .unwrap();
        if !matches {
            failures.push(reference.root);
        }
    }
    assert_ne!(
        native_values[0], native_values[1],
        "Native must select distinct physical slots"
    );
    assert!(
        failures.is_empty(),
        "mapped placeholder public output: {failures:?}"
    );
}
