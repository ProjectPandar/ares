use super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    stages::Stage,
};
use serde_json::{Value, json};
use std::time::Duration;

#[path = "../../../enum_declarations_support/mod.rs"]
mod support;

fn check_wire(key: &str, value: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let evidence = support::artifacts(&format!("wire-{key}-{}", value.replace(' ', "_")));
    let input = support::input(key, value);
    let path = runner.work.join("enum-state.3mf");
    std::fs::write(&path, &input).unwrap();
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
    let export = runner.work.join("native-settings.json");
    command::run(
        runner.bin(),
        [
            "--export-settings".to_owned(),
            export.to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        &[("LC_ALL", "C")],
        Limits {
            stage: Stage::Export,
            work: &runner.work,
            timeout: Duration::from_secs(120),
        },
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let native: Value = serde_json::from_slice(&std::fs::read(export).unwrap()).unwrap();
    assert_eq!(native["version"], "2.5.0");
    let project = ares_core::load_project(&input).unwrap();
    let actual = support::loaded_value(&project, key);
    std::fs::write(
        evidence.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "key": key, "authored": value, "native": native.get(key),
            "actual": actual, "native_work": runner.work,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        native.get(key).is_some(),
        "Native export omits {key}: {:?}",
        runner.work
    );
    assert_eq!(native[key], json!(value), "{key}: {:?}", runner.work);
    assert_eq!(actual, native[key], "{key}: {:?}", runner.work);
    let decoded = ares_core::decode_config_value_by_key(key, value).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&decoded).unwrap(),
        native[key]
    );
    assert_eq!(
        ares_core::encode_config_value_by_key(key, &decoded).unwrap(),
        value
    );
    assert!(
        !ares_core::project_config_substitutions(&input)
            .unwrap()
            .iter()
            .any(|entry| entry.key == key)
    );
}

macro_rules! wire_cases {
    ($($name:ident: $key:literal, $value:literal;)*) => {$(
        #[test]
        fn $name() {
            check_wire($key, $value);
        }
    )*};
}

wire_cases! {
    top_spiral_inset: "top_surface_pattern", "spiralinset";
    bottom_spiral_inset: "bottom_surface_pattern", "spiralinset";
    solid_spiral_inset: "internal_solid_infill_pattern", "spiralinset";
    support_spiral_inset: "support_interface_pattern", "spiralinset";
    leading_edge: "brim_type", "leading_edge_only";
    nozzle_manual: "filament_map_mode", "Nozzle Manual";
    ultimaker: "host_type", "ultimaker";
    best_of: "print_order", "best_of";
    snake: "print_order", "snake";
}
