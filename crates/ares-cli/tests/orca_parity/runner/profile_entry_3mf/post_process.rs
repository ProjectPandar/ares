/// O5 real-case oracle: the MK3S fixture (exported from the Prusa
/// `process_common_mk3` chain carrying `post_process: [""]`) is rejected
/// by the native CLI with exit code -19 and by Ares with the same message.
#[test]
fn mk3s_post_process_rejection() {
    let Some(runner) = OrcaRunner::from_env()
        .unwrap() else {
        return;
    };
    let root = super::repo_root();
    let fixture =
        std::fs::read(root.join("tests/parity/validation/post-process-mk3s-postprocess.3mf"))
            .unwrap();
    std::fs::write(runner.work.join("o5-mk3s.3mf"), &fixture).unwrap();

    // Native: direct 3MF slice → exit -19 (CLI_POSTPROCESS_NOT_SUPPORTED).
    let output = runner.work.join("o5-reference");
    std::fs::create_dir(&output).unwrap();
    let result = std::fs::read(runner.work.join("o5-mk3s.3mf")).unwrap();
    let native = stages::run(
        &runner.bin,
        [
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            runner.work.join("o5-mk3s.3mf").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    );
    // The native exits 237 (= 256 - 19); assert rejection, not success.
    assert!(native.is_err(), "native must reject the post_process 3MF");

    // Ares: the validated entry returns the exact upstream message.
    let error = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &result,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("post_process is not supported under CLI."),
        "{error}"
    );
    // No G-code was produced.
    assert!(!output.join("plate_1.gcode").exists());
}
