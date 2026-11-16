//! ToolOrdering.cpp:1796–1805,3079–3090: used-only extruder maps and full nozzle maps.
use crate::runner::{OrcaRunner, native_api_cli::NativeCase};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const KEYS: [&str; 4] = [
    "filament_map",
    "filament_volume_map",
    "filament_nozzle_map",
    "filament_map_2",
];

fn project(used: usize) -> Vec<u8> {
    let source = include_bytes!("../../../../../../../tests/parity/arachne/default-prisms.3mf");
    let mut archive = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut member = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        if member.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in settings.as_object_mut().unwrap() {
                if key.starts_with("filament_") {
                    if let Some(values) = value.as_array_mut() {
                        assert_eq!(values.len(), 1, "single-filament source {key}");
                        values.resize(4, values[0].clone());
                    }
                }
            }
            for (key, value) in [
                ("printer_model", json!("Snapmaker U1")),
                ("nozzle_diameter", json!(vec!["0.4"; 4])),
                ("extruder_type", json!(vec!["Direct Drive"; 4])),
                ("extruder_colour", json!(vec!["#F2754E"; 4])),
                ("extruder_max_nozzle_count", json!(["1"])),
                ("nozzle_volume_type", json!(vec!["Standard"; 4])),
                ("print_extruder_id", json!(["1", "2", "3", "4"])),
                (
                    "print_extruder_variant",
                    json!(vec!["Direct Drive Standard"; 4]),
                ),
                ("filament_self_index", json!(["1", "2", "3", "4"])),
                ("filament_map", json!(vec!["1"; 4])),
                ("filament_nozzle_map", json!(vec!["0"; 4])),
                ("filament_volume_map", json!(vec!["0"; 4])),
                ("inherits_group", json!(vec![""; 6])),
                ("different_settings_to_system", json!(vec![""; 6])),
                ("flush_multiplier", json!(vec!["1"; 4])),
                ("machine_start_gcode", json!("G28\n")),
                ("machine_end_gcode", json!("M104 S0\nM140 S0\n")),
            ] {
                settings[key] = value;
            }
            settings["flush_volumes_matrix"] = Value::Array(
                (0..64)
                    .map(|index| {
                        json!(if index % 16 / 4 == index % 4 {
                            "0"
                        } else {
                            "280"
                        })
                    })
                    .collect(),
            );
            bytes = serde_json::to_vec(&settings).unwrap();
        } else if member.name() == "Metadata/model_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            let selector = "<metadata key=\"extruder\" value=\"1\"/>";
            assert_eq!(text.matches(selector).count(), 3);
            bytes = text
                .replace(
                    selector,
                    &format!("<metadata key=\"extruder\" value=\"{used}\"/>"),
                )
                .into_bytes();
        }
        output
            .start_file(member.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn fields(bytes: &[u8]) -> BTreeMap<String, String> {
    let values = std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .filter(|(key, _)| KEYS.contains(key))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(values.len(), KEYS.len());
    values
}

fn check(used: usize) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(used);
    let reference = NativeCase::slice(&runner, &project, "non-bbl-auto-unused-map-");
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = reference.root.join("ares.cli.gcode");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &output.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &output.stderr).unwrap();
    fs::write(
        reference.root.join("ares.cli.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert!(output.status.success(), "CLI: {:?}", reference.root);
    let cli = fs::read(cli_path).unwrap();
    let native_fields = fields(reference.native.as_bytes());
    let api_fields = fields(&api);
    let cli_fields = fields(&cli);
    let usage = reference
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap();
    let actual_used = usage
        .split(',')
        .enumerate()
        .filter(|(_, amount)| amount.trim().parse::<f64>().unwrap() > 0.0)
        .map(|(index, _)| index + 1)
        .collect::<Vec<_>>();
    fs::write(reference.root.join("map-comparison.json"), serde_json::to_vec_pretty(&json!({
        "used_logical_filament": used, "native_usage_mm": usage,
        "native": native_fields, "public_api": api_fields, "cli": cli_fields,
        "strict_whole_api": crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api),
        "strict_whole_cli": crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli),
        "whole_output_parity_claimed": false
    })).unwrap()).unwrap();
    assert_eq!(
        actual_used,
        [used],
        "Native logical usage: {:?}",
        reference.root
    );
    assert_eq!(native_fields["filament_nozzle_map"], "0,1,2,3");
    assert_eq!(
        native_fields["filament_map"],
        if used == 1 { "1,1,1,1" } else { "1,2,1,1" }
    );
    assert_eq!(
        api_fields, native_fields,
        "public API: {:?}",
        reference.root
    );
    assert_eq!(cli_fields, native_fields, "CLI: {:?}", reference.root);
}

#[test]
fn non_bbl_auto_map_keeps_three_unused_filament_assignments() {
    check(1);
}

#[test]
fn non_bbl_auto_map_changes_only_the_used_second_filament() {
    check(2);
}
