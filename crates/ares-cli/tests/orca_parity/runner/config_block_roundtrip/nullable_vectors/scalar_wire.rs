use super::super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use serde_json::{Value, json};
use std::{fs, time::Duration};

#[test]
fn native_existing_crosshatch_scalar_wire_reaches_public_settings() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let producer = cache_identity::identify(runner.bin()).unwrap();
    fs::write(
        runner.work.join("native-producer.json"),
        serde_json::to_vec_pretty(&producer.receipt).unwrap(),
    )
    .unwrap();
    let source =
        fs::read(repo_root().join("tests/parity/crosshatch-ordering/control.3mf")).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&source),
        "73aad5c16915a4769bc9f8bffd75e782011a5f5d2e1afa07578d65296346221e"
    );
    let authored = super::super::super::application::settings(&source).unwrap();
    let keys = [
        "enable_overhang_speed",
        "slowdown_for_curled_perimeters",
        "outer_wall_speed",
    ];
    assert_eq!(authored[keys[0]], "1");
    assert_eq!(authored[keys[1]], "0");
    assert_eq!(authored[keys[2]], "40");
    let input = runner.work.join("original.3mf");
    fs::write(&input, &source).unwrap();
    let mut exports = Vec::new();
    for repeat in 1..=2 {
        let output = runner.work.join(format!("native-{repeat}.json"));
        command::run(
            runner.bin(),
            vec![
                "--export-settings".to_owned(),
                output.to_string_lossy().into_owned(),
                input.to_string_lossy().into_owned(),
            ],
            &[("LC_ALL", "C")],
            Limits {
                stage: Stage::Export,
                work: &runner.work,
                timeout: Duration::from_secs(120),
            },
        )
        .unwrap();
        exports.push(fs::read(output).unwrap());
    }
    assert_eq!(exports[0], exports[1], "full Native export replay");
    assert_eq!(fs::read(&input).unwrap(), source);
    let native: Value = serde_json::from_slice(&exports[0]).unwrap();
    let mut observations = Vec::new();
    let mut failures = Vec::new();
    for key in keys {
        assert_eq!(native[key], json!([authored[key]]));
        let request = json!({key: authored[key]});
        let region =
            serde_json::from_value::<ares_core::ProcessRegionSourceOptions>(request.clone())
                .map(|value| serde_json::to_value(value).unwrap())
                .map_err(|error| error.to_string());
        let settings = serde_json::from_value::<ares_core::ProjectSettings>(request)
            .map(|value| serde_json::to_value(value.process.region).unwrap())
            .map_err(|error| error.to_string());
        let matches = region.as_ref().is_ok_and(|value| value[key] == native[key])
            && settings
                .as_ref()
                .is_ok_and(|value| value[key] == native[key]);
        if !matches {
            failures.push(key);
        }
        observations.push(
            json!({"key":key,"authored":authored[key],"native":native[key],
            "region":region,"settings":settings,"matches":matches}),
        );
    }
    let settings = serde_json::from_value::<ares_core::ProjectSettings>(Value::Object(authored));
    let artifact = runner.work.join("scalar-wire-public-settings.json");
    fs::write(
        &artifact,
        serde_json::to_vec_pretty(&json!({
            "input_sha256":cache_identity::raw_digest(&source),
            "observations":observations,"failures":failures,
            "full_original_settings_error":settings.as_ref().err().map(ToString::to_string),
            "source":"Config.cpp:596-611,626-695,981-983;Config.hpp:932-956,2016-2048",
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "scalar wire public DTO boundary: {failures:?}; {artifact:?}"
    );
    assert!(settings.is_ok(), "full original settings: {artifact:?}");
}
