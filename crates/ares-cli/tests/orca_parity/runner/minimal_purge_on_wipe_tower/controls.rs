//! Remaining `WipeTower2.cpp:2336-2390` cases, each checked against Native first.
use crate::runner::{OrcaRunner, application, cache_identity};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[test]
fn receiving_identity_preserves_the_existing_finishing_floor() {
    run("receiving-swap-finishing", Some([35, 5]), false);
}

#[test]
fn zero_receiving_minimum_leaves_the_planned_wipe_unchanged() {
    run("nonbinding-zero", Some([5, 0]), false);
}

#[test]
fn equal_receiving_minimum_leaves_the_planned_wipe_unchanged() {
    run("nonbinding-equality", Some([5, 5]), false);
}

#[test]
fn absent_minimum_uses_the_registered_fifteen_default() {
    run("registered-default15", None, false);
}

#[test]
fn semm_allocation_keeps_its_separate_receiving_reservation() {
    run("semm-allocation-control", Some([5, 35]), true);
}

fn run(label: &str, minimum: Option<[u8; 2]>, semm: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let supply = application::project_supply()
        .unwrap()
        .expect("matched project preset catalog");
    let original = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/parity/print-order-as-obj-list/class-default.3mf"
    ));
    let mut settings = application::settings(original).unwrap();
    let flag = if semm { "1" } else { "0" };
    for (key, value) in json!({
        "single_extruder_multi_material": flag,
        "purge_in_prime_tower": flag,
        "prime_volume": "5",
        "wipe_tower_filament": "1",
        "enable_filament_ramming": "0",
        "filament_multitool_ramming": ["0", "0"],
        "wipe_tower_wall_type": "rectangle",
    })
    .as_object()
    .unwrap()
    {
        settings.insert(key.clone(), value.clone());
    }
    settings.remove("filament_minimal_purge_on_wipe_tower");
    if let Some(values) = minimum {
        settings.insert(
            "filament_minimal_purge_on_wipe_tower".to_owned(),
            json!(values.map(|value| value.to_string())),
        );
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(original)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = serde_json::Map::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        } else {
            unchanged.insert(
                entry.name().to_owned(),
                json!(cache_identity::raw_digest(&bytes)),
            );
        }
        writer
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let input = writer.finish().unwrap().into_inner();
    let exported = runner.fixture_case(label, input).unwrap();
    let root = exported.project_path.parent().unwrap();
    let native = runner.slice_selected_plate_case(&exported, 1).unwrap();
    fs::write(root.join("native.gcode"), &native.reference).unwrap();
    fs::write(root.join("catalog.json"), &supply.catalog).unwrap();
    let selection = r#"{"from_project":true,"uptodate":false}"#;
    fs::write(root.join("selection.json"), selection).unwrap();
    // ConfigOptionVector::get_at (Config.hpp:676-679) reads the single default
    // value for T1 too; the absent option exports as `15`, not explicit `15,15`.
    let footer = minimum.map_or_else(|| "15".to_owned(), |[a, b]| format!("{a},{b}"));
    let text = std::str::from_utf8(&native.reference).unwrap();
    for expected in [
        format!("; filament_minimal_purge_on_wipe_tower = {footer}"),
        format!("; single_extruder_multi_material = {flag}"),
        format!("; purge_in_prime_tower = {flag}"),
        "; prime_volume = 5".to_owned(),
        "; wipe_tower_filament = 1".to_owned(),
        "; wipe_tower_type = type2".to_owned(),
        "; filament_diameter = 1.75,1.75".to_owned(),
    ] {
        assert!(
            text.lines().any(|line| line == expected),
            "Native premise {expected}; {root:?}"
        );
    }
    assert!(
        !text.lines().any(|line| line.starts_with("M82")),
        "relative E witness required"
    );
    let wipes = super::wipes(&native.reference);
    let receiving: Vec<_> = wipes.iter().filter(|(tool, _)| *tool == 1).collect();
    let floor = f64::from(minimum.map_or(15, |values| values[1]));
    assert!(!receiving.is_empty(), "Native never reaches T1; {root:?}");
    assert!(
        receiving.iter().all(|(_, volume)| *volume >= floor),
        "Native receiving floor not reached; {root:?}"
    );
    if minimum.is_some_and(|values| values[1] <= 5) {
        assert!(
            receiving.iter().all(|(_, volume)| *volume < 15.0),
            "Native nonbinding T1 unexpectedly uses a larger minimum; {root:?}"
        );
    }
    if minimum == Some([35, 5]) {
        let finishing: Vec<_> = wipes.iter().filter(|(tool, _)| *tool == 0).collect();
        assert!(
            !finishing.is_empty() && finishing.iter().all(|(_, volume)| *volume >= 35.0),
            "Native existing finishing floor absent; {root:?}"
        );
    }
    fs::write(
        root.join("native-premise.json"),
        serde_json::to_vec_pretty(&json!({
            "case": label,
            "input_sha256": cache_identity::raw_digest(&exported.project),
            "original_sha256": cache_identity::raw_digest(original),
            "unchanged_entries_sha256": unchanged,
            "catalog_sha256": supply.digest,
            "settings": settings,
            "minimum_explicit": minimum,
            "SEMM_and_purge_enabled": semm,
            "forced_finishing_tool": 0,
            "non_finishing_receiving_tool": 1,
            "wipes": wipes,
            "runner_work": runner.work,
        }))
        .unwrap(),
    )
    .unwrap();

    let overrides = ares_core::ConfigOverrides::default();
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &exported.project,
            ares_core::ProfileSliceRequest {
                filename: "input.3mf",
                catalog_json: &supply.catalog,
                selection_json: selection,
                plate: Some(1),
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(root.join("ares.api.gcode"), bytes).unwrap();
    }
    let cli_path = root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "--plate", "1", "-o"])
        .arg(&cli_path)
        .arg(&exported.project_path)
        .output()
        .unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    let api_difference = api.as_ref().ok().and_then(|bytes| {
        crate::golden::compare_ordered_bytes_generator_only(&native.reference, bytes).err()
    });
    let cli_bytes = cli.status.success().then(|| fs::read(&cli_path).unwrap());
    let cli_difference = cli_bytes.as_ref().and_then(|bytes| {
        crate::golden::compare_ordered_bytes_generator_only(&native.reference, bytes).err()
    });
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&json!({
        "case": label,
        "public_entry": "ares_core::slice_with_profiles",
        "api_error": api.as_ref().err().map(ToString::to_string),
        "api_sha256": api.as_ref().ok().map(|bytes| cache_identity::raw_digest(bytes)),
        "api_difference": api_difference,
        "cli_exit": cli.status.code(),
        "cli_sha256": cli_bytes.as_ref().map(|bytes| cache_identity::raw_digest(bytes)),
        "cli_difference": cli_difference,
        "native_premise": "PASS",
        "whole_output_pass": api.is_ok() && api_difference.is_none() && cli.status.success() && cli_difference.is_none(),
    })).unwrap()).unwrap();
    let api = api.unwrap_or_else(|error| panic!("API admission before bytes: {error:?}; {root:?}"));
    assert!(
        cli.status.success(),
        "CLI admission before bytes: {:?}; {root:?}",
        cli.stderr
    );
    crate::golden::compare_ordered_bytes_generator_only(&native.reference, &api)
        .unwrap_or_else(|difference| panic!("{difference}; {root:?}"));
    crate::golden::compare_ordered_bytes_generator_only(
        &native.reference,
        cli_bytes.as_ref().unwrap(),
    )
    .unwrap_or_else(|difference| panic!("{difference}; {root:?}"));
}
