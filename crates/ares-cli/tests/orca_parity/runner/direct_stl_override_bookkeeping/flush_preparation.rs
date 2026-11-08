//! CLI palette-growth preparation through the public byte/profile entry.

use super::{FILAMENT, MACHINE, OrcaRunner, PROCESS, cache_identity, profile_entry};
use crate::runner::{repo_root, stages};
use serde_json::{Value, json};
use std::fs;

fn check(name: &str, fields: Value, filament_count: usize) {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "WEMAKE3D").unwrap();
    let mut vendor = profile_entry::vendor(&profiles_root, "WEMAKE3D");
    let process = vendor["files"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .find(|profile| profile["type"] == "process" && profile["name"] == PROCESS)
        .unwrap();
    for (key, value) in fields.as_object().unwrap() {
        process[key] = value.clone();
    }
    let catalog = json!({
        "vendors": [profile_entry::vendor(&profiles_root, "OrcaFilamentLibrary"), vendor],
        "imports": [],
    });
    let selection = json!({"machine": MACHINE, "process": PROCESS,
        "filaments": vec![FILAMENT; filament_count]});
    for (kind, mut preset) in [
        ("machine", profiles.machine(MACHINE).unwrap()),
        ("process", profiles.process(PROCESS).unwrap()),
        ("filament", profiles.filament(FILAMENT).unwrap()),
    ] {
        if kind == "process" {
            preset.extend(fields.as_object().unwrap().clone());
        }
        profile_entry::write_json(&runner.work.join(format!("{kind}.json")), &json!(preset));
    }
    profile_entry::write_json(&runner.work.join("catalog.json"), &catalog);
    profile_entry::write_json(&runner.work.join("selection.json"), &selection);
    let input =
        fs::read(root.join("tests/parity/counterbore-partiallybridge/solid-plate.stl")).unwrap();
    fs::write(runner.work.join("input.stl"), &input).unwrap();
    let output = runner.work.join("native");
    fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            format!(
                "{};{}",
                runner.work.join("machine.json").display(),
                runner.work.join("process.json").display()
            ),
            "--load-filaments".to_owned(),
            vec![runner.work.join("filament.json").to_str().unwrap(); filament_count].join(";"),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            runner.work.join("input.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &input,
            ares_core::ProfileSliceRequest {
                filename: "input.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &ares_core::ConfigOverrides::default(),
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ));
    let actual = match actual {
        Ok(bytes) => bytes,
        Err(error) => {
            profile_entry::write_json(
                &runner.work.join("flush-preparation-result.json"),
                &json!({"case": name, "selection": selection, "fields": fields,
                    "native_sha256": cache_identity::raw_digest(&reference),
                    "ares_error": format!("{error:?}")}),
            );
            panic!("{name}: {error:?}; {:?}", runner.work);
        }
    };
    fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    let rows = |bytes: &[u8]| {
        let text = std::str::from_utf8(bytes).unwrap();
        let block = text.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
        let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
        [
            "filament_colour",
            "flush_volumes_matrix",
            "flush_multiplier",
        ]
        .map(|key| {
            block
                .lines()
                .find(|line| line.starts_with(&format!("; {key} = ")))
                .unwrap()
                .to_owned()
        })
    };
    let native_rows = rows(&reference);
    let actual_rows = rows(&actual);
    let comparison = crate::golden::compare_ordered_bytes_generator_only(&reference, &actual);
    profile_entry::write_json(
        &runner.work.join("flush-preparation-result.json"),
        &json!({
            "case": name, "selection": selection, "fields": fields,
            "native_sha256": cache_identity::raw_digest(&reference),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "native_rows": native_rows, "ares_rows": actual_rows,
            "row_equal": native_rows == actual_rows,
            "whole_equal": comparison.is_ok(), "whole_difference": comparison.as_ref().err(),
            "rust_entry": "ares_core::slice_with_profiles",
            "comparison": "complete ordered bytes; generated-by identity only",
        }),
    );
    assert_eq!(actual_rows, native_rows, "{name}; {:?}", runner.work);
    comparison.unwrap();
}

#[test]
fn absent_palette_triggers_preparation() {
    check("absent-palette", json!({}), 1);
}

#[test]
fn empty_palette_rebuilds_supplied_matrix() {
    check(
        "empty-palette",
        json!({"filament_colour": [],
        "flush_volumes_matrix": ["57.5", "123", "456", "78"]}),
        1,
    );
}

#[test]
fn short_palette_grows_preserving_prefix() {
    check("short-palette", json!({"filament_colour": ["#102030"]}), 2);
}

#[test]
fn complete_palette_retains_supplied_matrix_across_modes() {
    for mode in ["Default", "Saving", "Fast"] {
        check(
            mode,
            json!({"filament_colour": ["#102030"],
            "flush_volumes_matrix": ["57.5"], "flush_multiplier": ["0.3"],
            "flush_multiplier_fast": ["0.7"], "prime_volume_mode": mode}),
            1,
        );
    }
}

#[test]
fn nontrigger_single_filament_mismatch_is_not_repaired() {
    check(
        "nontrigger-mismatch",
        json!({"filament_colour": ["#102030"],
        "flush_volumes_matrix": ["57.5", "123", "456", "78"]}),
        1,
    );
}
