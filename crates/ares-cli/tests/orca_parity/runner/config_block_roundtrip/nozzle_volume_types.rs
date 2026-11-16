use super::super::{
    OrcaRunner, application, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use ares_core::{NozzleVolumeTypes, SliceOptions};
use serde_json::{Value, json};
use std::{
    io::{Cursor, Write},
    time::Duration,
};

const KEYS: &[&str] = &[
    "nozzle_volume_type",
    "default_nozzle_volume_type",
    "extruder_nozzle_volume_type",
];
// PrintConfig.hpp:619–630 / PrintConfig.cpp:724–732: 4 is reserved.
const TYPES: &[(&str, i32)] = &[
    ("Standard", 0),
    ("High Flow", 1),
    ("Hybrid", 2),
    ("TPU High Flow", 3),
    ("E3D High Flow", 5),
    ("Extra High Flow", 6),
];

#[test]
fn standard_nozzle_volume_type_matches_native() {
    check_state(TYPES[0]);
}

#[test]
fn high_flow_nozzle_volume_type_matches_native() {
    check_state(TYPES[1]);
}

#[test]
fn hybrid_nozzle_volume_type_matches_native() {
    check_state(TYPES[2]);
}

#[test]
fn tpu_high_flow_nozzle_volume_type_matches_native() {
    check_state(TYPES[3]);
}

#[test]
fn e3d_high_flow_nozzle_volume_type_matches_native() {
    check_state(TYPES[4]);
}

#[test]
fn extra_high_flow_nozzle_volume_type_matches_native() {
    check_state(TYPES[5]);
}

fn check_state((name, id): (&str, i32)) {
    let Some((runner, input, native)) = native_settings(KEYS, &[name]) else {
        return;
    };
    let loaded = ares_core::load_project(&input).unwrap();
    let settings = loaded.settings();
    for (key, actual) in [
        (KEYS[0], &settings.project.gcode.nozzle_volume_type),
        (
            KEYS[1],
            &settings.printer.remaining.default_nozzle_volume_type,
        ),
    ] {
        assert_eq!(native[key], json!([name]), "{:?}", runner.work);
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            native[key],
            "{key}: {:?}",
            runner.work
        );
        assert_eq!(actual.0.iter().map(|v| *v as i32).collect::<Vec<_>>(), [id]);
    }
    for key in KEYS {
        assert_eq!(native[*key], json!([name]), "{key}: {:?}", runner.work);
        check_codec(key, &native[*key]);
    }
    let substitutions = ares_core::project_config_substitutions(&input).unwrap();
    let substitutions: Vec<_> = substitutions
        .iter()
        .filter(|entry| KEYS.contains(&entry.key.as_str()))
        .collect();
    assert!(
        substitutions.is_empty(),
        "{substitutions:?}: {:?}",
        runner.work
    );

    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4, 0.4],
        "extruder_type": ["Direct Drive", "Direct Drive"],
        "nozzle_volume_type": [name, name],
        "printer_extruder_variant": [format!("Direct Drive {name}")]
    }))
    .unwrap();
    assert!(!options.is_using_different_extruders().unwrap());
    // PrintConfig.hpp:680–692: Hybrid is not a physical preset variant.
    if name != "Hybrid" {
        assert_eq!(
            options
                .get_index_for_extruder_no_id("Direct Drive", name, "printer_extruder_variant", 1,)
                .unwrap(),
            0
        );
    }
    std::fs::write(
        runner.work.join("nozzle-type-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "native": native,
            "current": settings.project.gcode.nozzle_volume_type,
            "default": settings.printer.remaining.default_nozzle_volume_type,
            "integer": id,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn shared_nozzle_type_map_is_wider_than_physical_gui_choices() {
    let names: Vec<_> = TYPES.iter().map(|(name, _)| *name).collect();
    let Some((runner, input, native)) = native_settings(&[KEYS[2]], &names) else {
        return;
    };
    assert_eq!(native[KEYS[2]], json!(names), "{:?}", runner.work);
    check_codec(KEYS[2], &native[KEYS[2]]);
    let typed: NozzleVolumeTypes = serde_json::from_value(native[KEYS[2]].clone()).unwrap();
    let ids: Vec<_> = typed.0.iter().map(|value| *value as i32).collect();
    assert_eq!(ids, TYPES.iter().map(|(_, id)| *id).collect::<Vec<_>>());
    let substitutions = ares_core::project_config_substitutions(&input).unwrap();
    assert!(
        !substitutions.iter().any(|entry| entry.key == KEYS[2]),
        "{substitutions:?}"
    );
    for key in &KEYS[..2] {
        let metadata = ares_core::option_metadata(key).unwrap();
        assert_eq!(metadata.enum_values, names);
        assert_eq!(metadata.enum_labels, names);
        assert_eq!(metadata.enum_keys, ids);
    }
    let physical = ares_core::option_metadata(KEYS[2]).unwrap();
    let physical_names: Vec<_> = names
        .iter()
        .copied()
        .filter(|name| *name != "Hybrid")
        .collect();
    let physical_ids: Vec<_> = TYPES
        .iter()
        .filter_map(|(name, id)| (*name != "Hybrid").then_some(*id))
        .collect();
    assert_eq!(physical.enum_values, physical_names);
    assert_eq!(physical.enum_labels, physical_names);
    assert_eq!(physical.enum_keys, physical_ids);
    std::fs::write(
        runner.work.join("nozzle-gui-schema.json"),
        ares_core::print_config_def_json(),
    )
    .unwrap();
}

#[test]
fn unknown_nozzle_volume_token_substitutes_the_entire_vector_like_native() {
    let Some((runner, input, native)) =
        native_settings(&KEYS[..2], &["High Flow", "Unknown Nozzle Type"])
    else {
        return;
    };
    let project = ares_core::load_project(&input).unwrap();
    let settings = project.settings();
    for (key, actual) in [
        (KEYS[0], &settings.project.gcode.nozzle_volume_type),
        (
            KEYS[1],
            &settings.printer.remaining.default_nozzle_volume_type,
        ),
    ] {
        assert_eq!(native[key], json!(["Standard"]), "{:?}", runner.work);
        assert_eq!(serde_json::to_value(actual).unwrap(), native[key]);
    }
    let substitutions = ares_core::project_config_substitutions(&input).unwrap();
    for key in &KEYS[..2] {
        let entry = substitutions
            .iter()
            .find(|entry| entry.key == *key)
            .unwrap();
        assert_eq!(entry.new_value, "Standard");
    }
}

fn check_codec(key: &str, native: &Value) {
    let wire = native
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>()
        .join(",");
    let decoded = ares_core::decode_config_value_by_key(key, &wire).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&decoded).unwrap(), *native);
    assert_eq!(
        ares_core::encode_config_value_by_key(key, &decoded).unwrap(),
        wire
    );
}

fn native_settings(keys: &[&str], values: &[&str]) -> Option<(OrcaRunner, Vec<u8>, Value)> {
    let runner = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}"))?;
    let input = std::fs::read(repo_root().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let mut settings = application::settings(&input).unwrap();
    for key in keys {
        settings.insert((*key).to_owned(), json!(values));
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    let input = output.finish().unwrap().into_inner();
    let fixture = runner.work.join("nozzle-types.3mf");
    std::fs::write(&fixture, &input).unwrap();
    let producer = cache_identity::identify(runner.bin()).unwrap_or_else(|error| panic!("{error}"));
    std::fs::write(
        runner.work.join("native-producer.json"),
        serde_json::to_vec_pretty(&producer.receipt).unwrap(),
    )
    .unwrap();
    std::fs::write(
        runner.work.join("input-sha256.txt"),
        cache_identity::raw_digest(&input),
    )
    .unwrap();
    let path = runner.work.join("native-settings.json");
    command::run(
        runner.bin(),
        [
            "--export-settings".to_owned(),
            path.to_string_lossy().into_owned(),
            fixture.to_string_lossy().into_owned(),
        ],
        &[("LC_ALL", "C")],
        Limits {
            stage: Stage::Export,
            work: &runner.work,
            timeout: Duration::from_secs(120),
        },
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let native: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(native["version"], "2.5.0", "{:?}", runner.work);
    Some((runner, input, native))
}
