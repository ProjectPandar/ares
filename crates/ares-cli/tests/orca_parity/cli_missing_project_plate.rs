use std::{fs, path::Path};

use crate::{
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, application::cli_command, stages},
};

#[test]
fn cli_exit_missing_project_plate_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("missing-project-plate-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/custom-gcode-overview/two-plates.3mf");
    let native_output = root.join("native");
    let ares_output = root.join("ares");
    fs::create_dir(&native_output).unwrap();
    fs::create_dir(&ares_output).unwrap();

    let native = stages::run(
        runner.bin(),
        [
            "--slice",
            "3",
            "--outputdir",
            native_output.to_str().unwrap(),
            project.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        stages::Stage::Slice,
        &root,
    )
    .unwrap_err();
    assert_eq!(native.kind, stages::FailureKind::Process, "{native}");
    let native = native.captured.unwrap();
    assert_eq!(native.code, Some(254), "{root:?}");
    assert_eq!(native.stdout, b"run found error, return -2, exit...\n");
    assert!(native.stderr.is_empty(), "{root:?}");
    let native_result = fs::read(native_output.join("result.json")).unwrap();
    let result: serde_json::Value = serde_json::from_slice(&native_result).unwrap();
    assert_eq!(result["return_code"], -2);
    assert_eq!(result["plate_index"], 0);
    assert_eq!(result["error_string"], "Invalid parameters to the slicer.");

    let ares = cli_command()
        .args(["slice", "--plate", "3", "-o"])
        .arg(ares_output.join("plate_3.gcode"))
        .arg(project)
        .output()
        .unwrap();
    fs::write(root.join("ares.stdout"), &ares.stdout).unwrap();
    fs::write(root.join("ares.stderr"), &ares.stderr).unwrap();
    fs::write(
        root.join("ares.exit"),
        format!("{:?}\n", ares.status.code()),
    )
    .unwrap();
    for output in [&native_output, &ares_output] {
        assert!(
            fs::read_dir(output).unwrap().all(|file| {
                file.unwrap()
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "gcode")
            }),
            "unexpected G-code in {output:?}"
        );
    }
    assert_eq!(ares.status.code(), native.code, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(
        result_comparison_bytes(&fs::read(ares_output.join("result.json")).unwrap()),
        result_comparison_bytes(&native_result),
        "complete result.json: {root:?}",
    );
}
