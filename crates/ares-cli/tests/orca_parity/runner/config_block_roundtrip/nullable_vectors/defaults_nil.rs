use super::super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use serde_json::{Value, json};
use std::{fs, time::Duration};

const KEYS: [&str; 3] = [
    "enable_overhang_speed",
    "slowdown_for_curled_perimeters",
    "outer_wall_speed",
];

#[test]
fn native_three_vector_defaults_and_nil_preserve_public_shape() {
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
        ("pure-defaults", None),
        (
            "mixed-nil",
            Some(json!({
                "enable_overhang_speed": ["0", "nil", "1"],
                "slowdown_for_curled_perimeters": ["1", "nil", "0"],
                "outer_wall_speed": ["87", "nil", "113"],
            })),
        ),
        (
            "all-nil",
            Some(json!({
                "enable_overhang_speed": ["nil"],
                "slowdown_for_curled_perimeters": ["nil"],
                "outer_wall_speed": ["nil"],
            })),
        ),
    ];
    let mut observations = Vec::new();
    let mut failures = Vec::new();
    for (label, updates) in cases {
        let work = runner.work.join(label);
        fs::create_dir(&work).unwrap();
        let project = updates
            .as_ref()
            .map(|updates| super::input::settings_variant(&source, updates.as_object().unwrap()));
        let input = work.join("input.3mf");
        if let Some(project) = &project {
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
                    work: &work,
                    timeout: Duration::from_secs(120),
                },
            )
            .unwrap();
            exports.push(fs::read(output).unwrap());
        }
        assert_eq!(
            exports[0], exports[1],
            "full Native export must replay: {work:?}"
        );
        let native: Value = serde_json::from_slice(&exports[0]).unwrap();
        assert_eq!(native["version"], "2.5.0");
        for key in KEYS {
            assert!(native[key].is_array(), "{key}: {work:?}");
        }
        let typed = if project.is_none() {
            Ok(serde_json::to_value(ares_core::ProcessRegionSourceOptions::default()).unwrap())
        } else {
            let canonical = json!({
                "enable_overhang_speed": native[KEYS[0]],
                "slowdown_for_curled_perimeters": native[KEYS[1]],
                "outer_wall_speed": native[KEYS[2]],
            });
            serde_json::from_value::<ares_core::ProcessRegionSourceOptions>(canonical)
                .map(|options| serde_json::to_value(options).unwrap())
                .map_err(|error| error.to_string())
        };
        let loaded = project.as_ref().map(|project| {
            ares_core::load_project(project.clone())
                .map(|project| serde_json::to_value(&project.settings().process.region).unwrap())
                .map_err(|error| format!("{error:?}"))
        });
        for key in KEYS {
            let matches = typed.as_ref().is_ok_and(|typed| typed[key] == native[key]);
            let loaded_matches = loaded.as_ref().is_none_or(|loaded| {
                loaded
                    .as_ref()
                    .is_ok_and(|loaded| loaded[key] == native[key])
            });
            if !(matches && loaded_matches) {
                failures.push(format!("{label}:{key}"));
            }
        }
        observations.push(json!({
            "case": label, "requested": updates, "native": {
                "enable_overhang_speed": native[KEYS[0]],
                "slowdown_for_curled_perimeters": native[KEYS[1]],
                "outer_wall_speed": native[KEYS[2]],
            }, "typed": typed, "loaded": loaded,
        }));
    }
    let artifact = runner.work.join("three-vector-default-nil.json");
    fs::write(
        &artifact,
        serde_json::to_vec_pretty(&json!({
            "observations": observations, "failures": failures,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "default/nil public contracts: {failures:?}; {artifact:?}"
    );
}
