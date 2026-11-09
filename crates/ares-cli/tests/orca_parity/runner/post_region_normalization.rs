//! PrintApply.cpp:2208-2216: sparse-region tool use must retain the prime tower.
//! Failure inventory: shell-only tool count, lost prime flag, absent tower,
//! invalid Native placement, changed model payload, whole-output divergence.

use crate::runner::{OrcaRunner, native_api_cli::NativeCase, native_api_cli::sha};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const BASE: &[u8] = include_bytes!("../../../../../tests/parity/semm-ramming/R1.3mf");

fn input() -> (Vec<u8>, Value) {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut members = Vec::new();
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let before = sha(&bytes);
        match file.name() {
            "Metadata/project_settings.config" => {
                let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(settings["enable_prime_tower"], "1");
                assert_eq!(settings["single_extruder_multi_material"], "1");
                assert_eq!(settings["filament_colour"].as_array().unwrap().len(), 2);
                assert_eq!(settings["sparse_infill_density"], "15%");
                settings["enable_prime_tower"] = json!("1");
                settings["sparse_infill_filament_id"] = json!("2");
                bytes = serde_json::to_vec_pretty(&settings).unwrap();
            }
            "Metadata/model_settings.config" => {
                let model = String::from_utf8(bytes).unwrap();
                let old = "<metadata key=\"extruder\" value=\"2\"/>";
                assert_eq!(model.matches(old).count(), 1);
                bytes = model
                    .replace(old, "<metadata key=\"extruder\" value=\"1\"/>")
                    .into_bytes();
            }
            _ => {}
        }
        members.push(json!({
            "name":file.name(), "before_sha256":before, "after_sha256":sha(&bytes)
        }));
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let project = writer.finish().unwrap().into_inner();
    let mut after = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    assert_eq!(reader.len(), after.len());
    for index in 0..reader.len() {
        let mut original = reader.by_index(index).unwrap();
        let mut actual = after.by_index(index).unwrap();
        assert_eq!(original.name(), actual.name());
        assert_eq!(original.compression(), actual.compression());
        let mut a = Vec::new();
        let mut b = Vec::new();
        original.read_to_end(&mut a).unwrap();
        actual.read_to_end(&mut b).unwrap();
        match original.name() {
            "Metadata/project_settings.config" => {
                let old: Value = serde_json::from_slice(&a).unwrap();
                let mut new: Value = serde_json::from_slice(&b).unwrap();
                assert_eq!(new["enable_prime_tower"], "1");
                assert_eq!(new["sparse_infill_filament_id"], "2");
                new["sparse_infill_filament_id"] = old["sparse_infill_filament_id"].clone();
                assert_eq!(old, new);
            }
            "Metadata/model_settings.config" => {
                assert_eq!(
                    String::from_utf8(a)
                        .unwrap()
                        .replace(
                            "<metadata key=\"extruder\" value=\"2\"/>",
                            "<metadata key=\"extruder\" value=\"1\"/>"
                        )
                        .as_bytes(),
                    b
                );
            }
            _ => assert_eq!(a, b, "{}", original.name()),
        }
    }
    let binding = json!({
        "base_fixture":"tests/parity/semm-ramming/R1.3mf",
        "base_sha256":sha(BASE), "input_sha256":sha(&project),
        "placement":"stock model transforms and tower placement unchanged",
        "members":members
    });
    (project, binding)
}

#[test]
fn stock_sparse_tool_enables_prime_tower_whole_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let (project, binding) = input();
    let case = NativeCase::slice(&runner, &project, "post-region-prime-tower-");
    let result: Value =
        serde_json::from_slice(&fs::read(case.root.join("native/result.json")).unwrap()).unwrap();
    let tower_blocks = case.native.matches("; CP TOOLCHANGE UNLOAD\n").count();
    fs::write(
        case.root.join("input-binding.json"),
        serde_json::to_vec_pretty(&json!({
            "input":binding, "native_result":result, "native_tower_blocks":tower_blocks
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result["return_code"], 0, "{:?}", case.root);
    assert!(
        tower_blocks > 0,
        "Native must emit the tower: {:?}",
        case.root
    );
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}
