use super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

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

fn native_defaults() -> Option<(OrcaRunner, Value)> {
    let runner = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}"))?;
    let fixture = repo_root().join("tests/parity/orca_cli_ender3.3mf");
    let input = std::fs::read(&fixture).unwrap();
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
    assert_eq!(native["version"], "2.5.0-beta", "{:?}", runner.work);
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
