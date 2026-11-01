use super::super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use serde_json::{Value, json};
use std::{fs, time::Duration};

#[test]
fn native_scalar_authored_region_inputs_match_public_project_import() {
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
    let mut observations = Vec::new();
    let mut failures = Vec::new();
    for (key, scalar) in [
        ("enable_overhang_speed", "1"),
        ("slowdown_for_curled_perimeters", "1"),
        ("outer_wall_speed", "120"),
    ] {
        let work = runner.work.join(key);
        fs::create_dir(&work).unwrap();
        let mut updates = json!({
            "enable_overhang_speed": ["1"],
            "slowdown_for_curled_perimeters": ["0"],
            "outer_wall_speed": ["120"],
        });
        updates[key] = json!(scalar);
        let authored = super::input::settings_variant(&source, updates.as_object().unwrap());
        let input = work.join("scalar-authored.3mf");
        fs::write(&input, &authored).unwrap();
        let mut exports = Vec::new();
        for repeat in 1..=2 {
            let output = work.join(format!("native-{repeat}.json"));
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
                    work: &work,
                    timeout: Duration::from_secs(120),
                },
            )
            .unwrap();
            exports.push(fs::read(output).unwrap());
        }
        assert_eq!(exports[0], exports[1], "Native replay: {work:?}");
        assert_eq!(fs::read(&input).unwrap(), authored);
        let native: Value = serde_json::from_slice(&exports[0]).unwrap();
        assert_eq!(
            native[key],
            json!([scalar]),
            "Native scalar-input acceptance"
        );
        // Crucially import the original scalar-authored bytes, NOT Native's export.
        let loaded = ares_core::load_project(authored.clone())
            .map(|project| serde_json::to_value(&project.settings().process.region).unwrap())
            .map_err(|error| format!("{error:?}"));
        let matches = loaded
            .as_ref()
            .is_ok_and(|loaded| loaded[key] == native[key]);
        if !matches {
            failures.push(key);
        }
        observations.push(json!({
            "key": key, "authored": updates[key], "native": native[key],
            "loaded": loaded, "matches": matches,
            "input_sha256": cache_identity::raw_digest(&authored),
            "ares_imports_original_authored_bytes": true,
        }));
    }
    let artifact = runner.work.join("scalar-authored-comparison.json");
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
        "real scalar-input parsing residual: {failures:?}; {artifact:?}"
    );
}
