//! PrintConfig.cpp:5957-5966 / PrintConfig.hpp:1546 at 345efec64:
//! inner_wall_speed is a nullable vector, not a scalar compatibility shell.
use super::super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};

const KEY: &str = "inner_wall_speed";

fn native_export(runner: &OrcaRunner, work: &Path, project: Option<&[u8]>) -> Value {
    let input = work.join("input.3mf");
    if let Some(project) = project {
        fs::write(&input, project).unwrap();
    }
    let mut exports = Vec::new();
    for repeat in 1..=2 {
        let output = work.join(format!("native-{repeat}.json"));
        let mut args = vec![
            "--export-settings".to_owned(),
            output.to_string_lossy().into_owned(),
        ];
        if project.is_some() {
            args.push(input.to_string_lossy().into_owned());
        }
        command::run(
            runner.bin(),
            args,
            &[("LC_ALL", "C")],
            Limits {
                stage: Stage::Export,
                work,
                timeout: Duration::from_secs(120),
            },
        )
        .unwrap();
        exports.push(fs::read(output).unwrap());
    }
    assert_eq!(exports[0], exports[1], "complete Native replay: {work:?}");
    if let Some(project) = project {
        assert_eq!(fs::read(input).unwrap(), project);
    }
    let native: Value = serde_json::from_slice(&exports[0]).unwrap();
    assert_eq!(native["version"], "2.5.0-beta");
    assert!(native[KEY].is_array(), "Native vector premise: {work:?}");
    native
}

#[test]
fn native_inner_wall_speed_preserves_public_vector_contracts() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let producer = cache_identity::identify(runner.bin()).unwrap();
    fs::write(
        runner.work.join("native-producer.json"),
        serde_json::to_vec_pretty(&producer.receipt).unwrap(),
    )
    .unwrap();
    let source = fs::read(repo_root().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let cases = [
        ("pure-default", None),
        ("singleton", Some(json!(["73"]))),
        ("distinct-columns", Some(json!(["19", "23", "37", "53"]))),
        ("mixed-nil", Some(json!(["87", "nil", "113"]))),
        ("all-nil", Some(json!(["nil"]))),
        ("scalar-wire", Some(json!("73"))),
        ("comma-wire", Some(json!("73,89"))),
    ];
    let mut observations = Vec::new();
    let mut failures = Vec::new();
    for (label, authored) in cases {
        let work = runner.work.join(label);
        fs::create_dir(&work).unwrap();
        let request = authored.as_ref().map(|value| json!({KEY: value}));
        let project = request
            .as_ref()
            .map(|request| super::input::settings_variant(&source, request.as_object().unwrap()));
        // All public observations follow two successful, identical Native exports.
        let native = native_export(&runner, &work, project.as_deref());
        let expected_native = authored.as_ref().map_or_else(
            || json!(["60"]),
            |value| match value {
                Value::String(wire) => json!(wire.split(',').collect::<Vec<_>>()),
                Value::Array(_) => value.clone(),
                _ => unreachable!(),
            },
        );
        assert_eq!(
            native[KEY], expected_native,
            "Native authored premise: {work:?}"
        );
        let region = request.as_ref().map_or_else(
            || Ok(serde_json::to_value(ares_core::ProcessRegionSourceOptions::default()).unwrap()),
            |request| {
                serde_json::from_value::<ares_core::ProcessRegionSourceOptions>(request.clone())
                    .map(|options| serde_json::to_value(options).unwrap())
                    .map_err(|error| error.to_string())
            },
        );
        let settings = request.as_ref().map_or_else(
            || {
                Ok(
                    serde_json::to_value(ares_core::ProjectSettings::default().process.region)
                        .unwrap(),
                )
            },
            |request| {
                serde_json::from_value::<ares_core::ProjectSettings>(request.clone())
                    .map(|settings| serde_json::to_value(settings.process.region).unwrap())
                    .map_err(|error| error.to_string())
            },
        );
        let options = request.as_ref().map(|request| {
            serde_json::from_value::<ares_core::SliceOptions>(request.clone())
                .map(|options| serde_json::to_value(options.values()).unwrap())
                .map_err(|error| error.to_string())
        });
        // Original input bytes, never Native's export, enter project import.
        let loaded = project.as_ref().map(|project| {
            ares_core::load_project(project.as_slice())
                .map(|project| serde_json::to_value(&project.settings().process.region).unwrap())
                .map_err(|error| format!("{error:?}"))
        });
        let token = native[KEY]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(",");
        let expected_typed = Value::Array(
            native[KEY]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    let value = value.as_str().unwrap();
                    if value == "nil" {
                        Value::Null
                    } else {
                        json!(value.parse::<f64>().unwrap())
                    }
                })
                .collect(),
        );
        let decoded = ares_core::decode_config_value_by_key(KEY, &token);
        let encoded = decoded
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|value| ares_core::encode_config_value_by_key(KEY, value));
        let decoded_value = decoded
            .as_ref()
            .ok()
            .map(|value| serde_json::from_str::<Value>(value).unwrap());
        let schema: Value = serde_json::from_slice(ares_core::print_config_def_json()).unwrap();
        let definition = schema["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|definition| definition["key"] == KEY)
            .unwrap();
        let matches = |result: &Result<Value, String>| {
            result.as_ref().is_ok_and(|value| value[KEY] == native[KEY])
        };
        let checks = json!({
            "region_full_array": matches(&region), "settings_full_array": matches(&settings),
            "options_full_array": options.as_ref().is_none_or(matches),
            "original_input_full_array": loaded.as_ref().is_none_or(matches),
            "codec_full_array": decoded_value.as_ref() == Some(&expected_typed),
            "wire_exact": encoded.as_deref() == Ok(token.as_str()),
            "definition_kind": definition["type"] == "floats_nullable",
            "definition_nullable": definition["nullable"] == true,
            "definition_default": label != "pure-default" || definition["default_value"] == token,
        });
        for (check, passed) in checks.as_object().unwrap() {
            if passed != true {
                failures.push(format!("{label}:{check}"));
            }
        }
        observations.push(json!({
            "case": label, "authored": authored, "native_full_array": native[KEY],
            "source_sha256": cache_identity::raw_digest(&source),
            "input_sha256": project.as_ref().map(|bytes| cache_identity::raw_digest(bytes)),
            "region": region, "settings": settings, "options": options, "loaded": loaded,
            "native_wire": token, "expected_typed_array": expected_typed,
            "decoded": decoded, "encoded": encoded, "public_definition": definition, "checks": checks,
        }));
    }
    let artifact = runner
        .work
        .join("inner-wall-speed-public-vector-contracts.json");
    fs::write(&artifact, serde_json::to_vec_pretty(&json!({
        "observations": observations, "failures": failures,
        "whole_output_or_runtime_index_parity_claimed": false,
        "upstream": "345efec64c7b95a05ce5cf96df0dde66a158835c PrintConfigDef/PrintRegionConfig inner_wall_speed",
    })).unwrap()).unwrap();
    assert!(
        failures.is_empty(),
        "inner-wall speed public vector contracts: {failures:?}; {artifact:?}"
    );
}
