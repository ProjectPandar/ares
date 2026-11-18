//! M854 oracle: public `ConfigOverrides` must match official OrcaSlicer
//! receiving the same edits through real CLI options, including dirty keys
//! (`Config.cpp:265-282 cli_args`, `read_cli:1780-1920`,
//! `OrcaSlicer.cpp:8203-8209`, `PresetBundle::full_config`).

use serde_json::{Value, json};

use super::{OrcaRunner, stages};

fn overrides() -> Vec<(&'static str, Value)> {
    vec![
        ("layer_height", json!("0.3")),
        ("wall_loops", json!("4")),
        ("sparse_infill_density", json!("30%")),
        ("enable_support", json!("1")),
        // Vector overrides use the upstream `serialize()` comma token
        // (inverse of `decodeConfigValue`); no per-index writes.
        ("nozzle_temperature", json!("210,220")),
    ]
}

fn overrides_json() -> String {
    let mut object = serde_json::Map::new();
    for (key, value) in overrides() {
        object.insert(key.to_owned(), value);
    }
    serde_json::to_string(&object).unwrap()
}

#[test]
fn user_overrides_match_native_cli_option_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        std::fs::read(super::repo_root().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let project_path = runner.work.join("m854-user-overrides.3mf");
    let output_dir = runner.work.join("m854-user-overrides-orca");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let mut args = Vec::new();
    for (key, value) in overrides() {
        args.push(format!("--{}", key.replace('_', "-")));
        if key != "enable_support" {
            args.push(value.as_str().unwrap().to_owned());
        }
    }
    args.extend([
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output_dir.to_str().unwrap().to_owned(),
        project_path.to_str().unwrap().to_owned(),
    ]);
    stages::run(runner.bin(), args, stages::Stage::Slice, &runner.work).unwrap();
    let reference = std::fs::read(output_dir.join("plate_1.gcode")).unwrap();

    let overrides = ares_core::ConfigOverrides::from_json(overrides_json().as_bytes()).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &overrides,
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();

    let outcome = crate::golden::compare_ordered_bytes_generator_only(&reference, &actual);
    assert!(
        outcome.is_ok(),
        "overridden slice diverged from Native CLI option output: {outcome:?}"
    );
}
