#[path = "definition_defaults/preset_catalog.rs"]
mod preset_catalog;

use super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    io::{Cursor, Write},
    time::Duration,
};

#[derive(Deserialize)]
struct Schema {
    options: Vec<Definition>,
}

#[derive(Deserialize)]
struct Definition {
    key: String,
    #[serde(rename = "type")]
    kind: String,
    default_value: String,
}

pub(super) fn native_defaults() -> Option<(OrcaRunner, Value)> {
    native_settings(None)
}

fn native_settings(setting: Option<(&str, &str)>) -> Option<(OrcaRunner, Value)> {
    let runner = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}"))?;
    let mut fixture = repo_root().join("tests/parity/orca_cli_ender3.3mf");
    let mut input = std::fs::read(&fixture).unwrap();
    if let Some((key, token)) = setting {
        let mut settings = super::super::application::settings(&input).unwrap();
        settings.insert(key.to_owned(), Value::String(token.to_owned()));
        let mut archive = zip::ZipArchive::new(Cursor::new(&input)).unwrap();
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
        input = output.finish().unwrap().into_inner();
        fixture = runner.work.join("legacy-settings-input.3mf");
        std::fs::write(&fixture, &input).unwrap();
    }
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
    let settings = runner.work.join("native-default-settings.json");
    command::run(
        runner.bin(),
        [
            "--export-settings".to_owned(),
            settings.to_string_lossy().into_owned(),
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
    let native: Value = serde_json::from_slice(&std::fs::read(settings).unwrap()).unwrap();
    assert_eq!(native["version"], "2.5.0", "{:?}", runner.work);
    std::fs::write(
        runner.work.join("ares-print-config-def.json"),
        ares_core::print_config_def_json(),
    )
    .unwrap();
    Some((runner, native))
}

fn exported_definition(runner: &OrcaRunner, key: &str) -> Definition {
    let schema: Schema = serde_json::from_slice(ares_core::print_config_def_json()).unwrap();
    schema
        .options
        .into_iter()
        .find(|definition| definition.key == key)
        .unwrap_or_else(|| {
            panic!(
                "Ares public PrintConfigDef is missing {key}; Native settings and Ares export: {:?}",
                runner.work
            )
        })
}

/// Config.hpp:837-842 uses a fresh stream, as format.hpp's optionless Boost arguments do.
/// Compare the real Native scalar text with the public encoder, not a private formatter.
#[test]
fn native_default_double_stream_spelling_matches_public_encoder() {
    let mut samples = Vec::new();
    for input in [
        "10",
        "0.5",
        "1000000",
        "0.00001",
        "0.0001",
        "123456.789",
        "999999.5",
        "3.4028234663852886e38",
    ] {
        let Some((runner, settings)) = native_settings(Some(("time_cost", input))) else {
            return;
        };
        let native = settings["time_cost"]
            .as_str()
            .expect("Native coFloat settings export is its serialize() string");
        let encoded = ares_core::encode_config_value_by_key("time_cost", input).unwrap();
        let sample = serde_json::json!({
            "input": input, "native": native, "ares": encoded,
            "artifacts": runner.work,
            "source": "ConfigOptionFloat::serialize; default double stream shared by Boost %N%",
        });
        std::fs::write(
            runner.work.join("double-stream-spelling.json"),
            serde_json::to_vec_pretty(&sample).unwrap(),
        )
        .unwrap();
        assert_eq!(encoded, native, "default double stream: {:?}", runner.work);
        samples.push(sample);
    }
    std::fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join("double-stream-equivalence.json"),
        serde_json::to_vec_pretty(&samples).unwrap(),
    )
    .unwrap();
}

#[test]
fn native_incomplete_exponent_float_vector_matches_public_decode_encode() {
    let key = "machine_max_speed_x";
    let input = "1e+";
    let Some((runner, settings)) = native_settings(Some((key, input))) else {
        return;
    };
    let [native] = settings[key].as_array().unwrap().as_slice() else {
        panic!(
            "Native single input element changed shape: {:?}",
            runner.work
        );
    };
    let native = native.as_str().unwrap();
    let decoded = ares_core::decode_config_value_by_key(key, input);
    let encoded = match &decoded {
        Ok(value) => ares_core::encode_config_value_by_key(key, value),
        Err(error) => Err(error.to_owned()),
    };
    std::fs::write(
        runner.work.join("incomplete-exponent-public-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "key": key, "input": input, "native": native,
            "public_decoded": decoded, "public_encoded": encoded,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        encoded.unwrap(),
        native,
        "Native vector extraction versus public decode/encode: {:?}",
        runner.work
    );
}

#[test]
fn native_float_or_percent_vector_retains_failed_value_and_percent() {
    let key = "small_support_perimeter_speed";
    let input = "bad%";
    let Some((runner, settings)) = native_settings(Some((key, input))) else {
        return;
    };
    let [native] = settings[key].as_array().unwrap().as_slice() else {
        panic!(
            "Native single input element changed shape: {:?}",
            runner.work
        );
    };
    let native = native.as_str().unwrap();
    let decoded = ares_core::decode_config_value_by_key(key, input);
    let encoded = match &decoded {
        Ok(value) => ares_core::encode_config_value_by_key(key, value),
        Err(error) => Err(error.to_owned()),
    };
    std::fs::write(
        runner
            .work
            .join("float-or-percent-vector-public-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "key": key, "input": input, "native": native,
            "public_decoded": decoded, "public_encoded": encoded,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        encoded,
        Ok(native.to_owned()),
        "Native vector extraction versus public decode/encode: {:?}",
        runner.work
    );
}

#[test]
fn native_arc_fitting_enum_decodes_and_round_trips_public_options() {
    let Some((runner, native)) = native_defaults() else {
        return;
    };
    let token = native["enable_arc_fitting"].as_str().unwrap();
    assert_eq!(token, "disabled");
    let decoded = ares_core::decode_config_value_by_key("enable_arc_fitting", token)
        .unwrap_or_else(|error| panic!("public arc enum decode: {error}; {:?}", runner.work));
    assert_eq!(
        decoded,
        serde_json::to_string(&native["enable_arc_fitting"]).unwrap()
    );
    let options: ares_core::ProcessGCodeSourceOptions = serde_json::from_value(
        serde_json::json!({"enable_arc_fitting": native["enable_arc_fitting"]}),
    )
    .unwrap_or_else(|error| panic!("public typed arc option: {error}; {:?}", runner.work));
    let exported = serde_json::to_value(options).unwrap();
    std::fs::write(
        runner.work.join("ares-arc-options.json"),
        serde_json::to_vec_pretty(&exported).unwrap(),
    )
    .unwrap();
    assert_eq!(exported["enable_arc_fitting"], native["enable_arc_fitting"]);
    assert_eq!(
        ares_core::encode_config_value_by_key("enable_arc_fitting", &decoded).unwrap(),
        token
    );
}

#[test]
fn legacy_tree_wall_count_normalizes_to_native_auto_options() {
    let key = "tree_support_wall_count";
    let Some((runner, native)) = native_settings(Some((key, "-1"))) else {
        return;
    };
    assert_eq!(native[key], "0", "Native legacy -1 must resolve to auto");
    let requested = serde_json::json!({key: "-1"});
    let options: ares_core::SliceOptions = serde_json::from_value(requested).unwrap();
    let project = ares_core::load_project(
        std::fs::read(runner.work.join("legacy-settings-input.3mf")).unwrap(),
    )
    .unwrap();
    let typed_export = serde_json::to_value(&project.settings().process.object).unwrap();
    std::fs::write(
        runner.work.join("ares-tree-options.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "slice_options": options.values(), "typed_options": typed_export,
            "native": native[key], "requested": "-1"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        options.values()[key],
        native[key],
        "public SliceOptions legacy normalization"
    );
    assert_eq!(
        typed_export[key], native[key],
        "public typed legacy normalization"
    );
}

#[test]
fn prime_volume_mode_default_matches_native_settings_export() {
    let Some((runner, native)) = native_defaults() else {
        return;
    };
    let native_default = native["prime_volume_mode"].as_str().unwrap();
    let definition = exported_definition(&runner, "prime_volume_mode");
    assert_eq!(definition.kind, "enum");
    assert_eq!(
        definition.default_value, native_default,
        "{:?}",
        runner.work
    );
}

#[test]
fn mixed_filament_flag_default_matches_native_settings_export() {
    let Some((runner, native)) = native_defaults() else {
        return;
    };
    let [native_default] = native["filament_is_mixed"].as_array().unwrap().as_slice() else {
        panic!("Native default must have one filament: {:?}", runner.work);
    };
    let native_default = native_default.as_str().unwrap();
    let definition = exported_definition(&runner, "filament_is_mixed");
    assert_eq!(definition.kind, "bools");
    assert_eq!(
        definition.default_value, native_default,
        "{:?}",
        runner.work
    );
}

#[test]
fn mixed_filament_components_default_matches_native_settings_export() {
    let Some((runner, native)) = native_defaults() else {
        return;
    };
    let [native_default] = native["filament_mixed_components"]
        .as_array()
        .unwrap()
        .as_slice()
    else {
        panic!("Native default must have one filament: {:?}", runner.work);
    };
    let native_default = native_default.as_str().unwrap();
    let definition = exported_definition(&runner, "filament_mixed_components");
    assert_eq!(definition.kind, "strings");
    assert_eq!(
        definition.default_value, native_default,
        "{:?}",
        runner.work
    );
}

#[test]
fn filament_prime_volume_default_matches_native_settings_export() {
    let Some((runner, native)) = native_defaults() else {
        return;
    };
    let [native_default] = native["filament_prime_volume"]
        .as_array()
        .unwrap()
        .as_slice()
    else {
        panic!("Native default must have one filament: {:?}", runner.work);
    };
    let native_default = native_default.as_str().unwrap();
    let definition = exported_definition(&runner, "filament_prime_volume");
    assert_eq!(definition.kind, "floats");
    assert_eq!(
        definition.default_value, native_default,
        "{:?}",
        runner.work
    );
}
