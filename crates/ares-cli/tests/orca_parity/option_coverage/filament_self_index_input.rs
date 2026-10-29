//! PresetBundle.cpp:172-195; PrintConfig.cpp:8375-8415: variant-slot payloads.
use super::*;
use crate::runner::ExportedCase;
use serde_json::json;
use std::io::{Cursor, Read, Write};

pub(super) fn project(runner: &OrcaRunner, label: &str, ids: &[&str]) -> ExportedCase {
    let mut exported = tool_context::mapping(runner, label, &json!(["1", "2"])).unwrap();
    let before = crate::runner::application::settings(&exported.project).unwrap();
    let slots = if ids.len() == 4 {
        [0, 0, 1, 1].to_vec()
    } else {
        vec![0, 1]
    };
    let mut fields = Map::new();
    for key in ares_core::filament_options_with_variant() {
        // Native export is the boundary: legacy set names absent from the
        // live config stay absent (PrintConfig.cpp:9873-9877,9894-9897).
        let Some(raw) = before.get(*key) else {
            continue;
        };
        let values = raw.as_array().unwrap();
        let expanded = slots
            .iter()
            .map(|&logical| {
                // ConfigOptionVector::get_at uses the first value past a singleton.
                values.get(logical).unwrap_or(&values[0]).clone()
            })
            .collect::<Vec<_>>();
        fields.insert((*key).into(), Value::Array(expanded));
    }
    let (variants, temperatures) = if ids.len() == 4 {
        (
            json!([
                "Direct Drive Standard",
                "Direct Drive High Flow",
                "Direct Drive Standard",
                "Direct Drive High Flow"
            ]),
            json!(["211", "213", "233", "235"]),
        )
    } else {
        (
            json!(["Direct Drive Standard", "Direct Drive Standard"]),
            json!(["211", "233"]),
        )
    };
    fields.insert("filament_self_index".into(), json!(ids));
    fields.insert("filament_extruder_variant".into(), variants);
    fields.insert("nozzle_temperature".into(), temperatures.clone());
    fields.insert("nozzle_temperature_initial_layer".into(), temperatures);
    let mut reader = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings.as_object_mut().unwrap().extend(fields.clone());
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let project = writer.finish().unwrap().into_inner();
    let mut after = crate::runner::application::settings(&project).unwrap();
    for (key, value) in &fields {
        assert_eq!(after[key], *value, "{label}: {key}");
        after.insert(key.clone(), before[key].clone());
    }
    assert_eq!(after, before, "{label}: unrelated settings changed");
    let mut patched = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    assert_eq!(reader.len(), patched.len());
    for index in 0..reader.len() {
        let mut original = reader.by_index(index).unwrap();
        if original.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut actual = patched.by_name(original.name()).unwrap();
        let mut a = Vec::new();
        let mut b = Vec::new();
        original.read_to_end(&mut a).unwrap();
        actual.read_to_end(&mut b).unwrap();
        assert_eq!(a, b, "{label}: {}", original.name());
    }
    exported.project = project;
    let result = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    for (key, value) in &fields {
        crate::runner::application::verify_value(&result.project, key, value).unwrap();
    }
    result
}
