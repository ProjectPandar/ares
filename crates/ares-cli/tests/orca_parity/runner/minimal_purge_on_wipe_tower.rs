//! `WipeTower2.cpp:2336-2390`: preserve the receiving minimum on non-finishing changes.
use crate::runner::{OrcaRunner, application, cache_identity};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[path = "minimal_purge_on_wipe_tower/controls.rs"]
mod controls;

fn wipes(gcode: &[u8]) -> Vec<(usize, f64)> {
    let mut tool = 0;
    let mut wiping = false;
    let mut extrusion = 0.0;
    let mut result = Vec::new();
    for line in std::str::from_utf8(gcode).unwrap().lines() {
        let command = line.split(';').next().unwrap().trim();
        if let Some(next) = command
            .strip_prefix('T')
            .and_then(|value| value.parse().ok())
        {
            tool = next;
        }
        if line == "; CP TOOLCHANGE WIPE" {
            wiping = true;
            extrusion = 0.0;
        }
        if wiping
            && command.starts_with("G1 ")
            && command
                .split_whitespace()
                .any(|word| word.starts_with(['X', 'Y']))
        {
            extrusion += command
                .split_whitespace()
                .find_map(|word| word.strip_prefix('E'))
                .map_or(0.0, |value| value.parse::<f64>().unwrap().max(0.0));
        }
        if wiping && line == "; WIPE_TOWER_END" {
            result.push((
                tool,
                extrusion * std::f64::consts::FRAC_PI_4 * 1.75_f64.powi(2),
            ));
            wiping = false;
        }
    }
    result
}

#[test]
fn non_finishing_receiving_filament_keeps_its_minimum_complete_output() {
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
    let changes = json!({
        "single_extruder_multi_material": "0",
        "purge_in_prime_tower": "0",
        "prime_volume": "5",
        "filament_minimal_purge_on_wipe_tower": ["5", "35"],
        "wipe_tower_filament": "1",
        "enable_filament_ramming": "0",
        "filament_multitool_ramming": ["0", "0"],
        "wipe_tower_wall_type": "rectangle",
    });
    for (key, value) in changes.as_object().unwrap() {
        settings.insert(key.clone(), value.clone());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(original)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged_entries = serde_json::Map::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        } else {
            unchanged_entries.insert(
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
    let exported = runner
        .fixture_case("minimum-purge-nonfinishing", input)
        .unwrap();
    let root = exported.project_path.parent().unwrap();
    let native = runner.slice_selected_plate_case(&exported, 1).unwrap();
    fs::write(root.join("native.gcode"), &native.reference).unwrap();
    fs::write(root.join("catalog.json"), &supply.catalog).unwrap();
    let selection = r#"{"from_project":true,"uptodate":false}"#;
    fs::write(root.join("selection.json"), selection).unwrap();
    let text = std::str::from_utf8(&native.reference).unwrap();
    for expected in [
        "; filament_minimal_purge_on_wipe_tower = 5,35",
        "; prime_volume = 5",
        "; purge_in_prime_tower = 0",
        "; single_extruder_multi_material = 0",
        "; wipe_tower_filament = 1",
        "; wipe_tower_type = type2",
        "; filament_diameter = 1.75,1.75",
    ] {
        assert!(
            text.lines().any(|line| line == expected),
            "Native premise {expected}; {root:?}"
        );
    }
    assert!(
        !text.lines().any(|line| line.starts_with("M82")),
        "Native witness requires the fixture's relative E mode"
    );
    let native_wipes = wipes(&native.reference);
    let receiving: Vec<_> = native_wipes.iter().filter(|(tool, _)| *tool == 1).collect();
    assert!(
        !receiving.is_empty(),
        "Native never wipes the non-finishing receiving T1; {root:?}"
    );
    assert!(
        receiving.iter().all(|(_, volume)| *volume >= 35.0),
        "Native did not enforce receiving minimum: {receiving:?}; {root:?}"
    );
    fs::write(
        root.join("native-premise.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": cache_identity::raw_digest(&exported.project),
            "original_sha256": cache_identity::raw_digest(original),
            "unchanged_entries_sha256": unchanged_entries,
            "changes": changes,
            "catalog_sha256": supply.digest,
            "selection": selection,
            "planned_volume_mm3": 5,
            "forced_finishing_tool": 0,
            "non_finishing_receiving_tool": 1,
            "minimum_mm3": 35,
            "wipes": native_wipes,
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
        "public_entry": "ares_core::slice_with_profiles",
        "api_error": api.as_ref().err().map(ToString::to_string),
        "api_difference": api_difference,
        "cli_exit": cli.status.code(),
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
