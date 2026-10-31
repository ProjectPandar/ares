use super::*;

#[test]
fn initialized_reference_repeats_complete_stock_project_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_OBJECT_ID_REFERENCE") {
        return;
    }
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() != Ok("1") {
        crate::native_environment::skipped("seeded-only reference");
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_eq!(std::env::var("ORCA_REPLAY_SEED").unwrap(), "0");
    let directory = tempfile::Builder::new()
        .prefix("initialized-object-id-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = runner::repo_root().join("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
    let project = std::fs::read(project).unwrap();
    crate::artifacts::write(&directory.join("input.3mf"), &project).unwrap();
    let settings = runner::application::settings(&project).unwrap();
    assert_eq!(settings["exclude_object"], "0");
    assert_eq!(settings["gcode_label_objects"], "1");
    crate::artifacts::write(
        &directory.join("reference.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "kind": "reference_self_replay",
            "contract_under_test": "seed_fill_and_object_id_initialization",
            "binary": std::env::var("ORCA_REPLAY_BIN").unwrap(),
            "seed": 0,
            "cpu": std::env::var("ORCA_REPLAY_CPU").unwrap(),
            "scope": "single-object KSR self-replay; not Ares parity or multi-object uniqueness",
        }))
        .unwrap(),
    )
    .unwrap();
    let mut outputs = Vec::new();
    for index in 0..2 {
        let output = directory.join(format!("run-{index}"));
        std::fs::create_dir(&output).unwrap();
        runner::stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                directory.join("input.3mf").to_str().unwrap().to_owned(),
            ],
            runner::stages::Stage::Slice,
            &output,
        )
        .unwrap();
        outputs.push(std::fs::read(output.join("plate_1.gcode")).unwrap());
    }
    let normalized = outputs
        .iter()
        .map(|output| {
            crate::golden::normalize_one_generator_line(output, crate::golden::GeneratorKind::Orca)
                .unwrap()
        })
        .collect::<Vec<_>>();
    if let Some(difference) = crate::golden::first_difference(&normalized[0], &normalized[1]) {
        panic!("reference self-replay differs: {difference}; complete artifacts {directory:?}");
    }
    let mut labels = std::str::from_utf8(&outputs[0])
        .unwrap()
        .lines()
        .filter(|line| {
            line.starts_with("; printing object ") || line.starts_with("; stop printing object ")
        })
        .peekable();
    assert!(labels.peek().is_some(), "object labels are not active");
    for label in labels {
        assert!(
            label.ends_with(" id:0 copy 0"),
            "unassigned single-object ID convention violated: {label}; artifacts {directory:?}"
        );
    }
}
