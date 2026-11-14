use crate::{OrcaRunner, golden, runner::application::cli_command, runner::stages};
use std::fs;
#[path = "auto_orientation/coverage.rs"]
mod coverage;
#[path = "auto_orientation/fixtures.rs"]
mod fixtures;
#[path = "auto_orientation/fresh.rs"]
mod fresh;
use fixtures::asymmetric_project;
#[test]
fn default_keeps_saved_mesh_like_native() {
    check(None);
}

#[test]
fn zero_keeps_saved_mesh_like_native() {
    check(Some("0"));
}

#[test]
fn one_orients_asymmetric_mesh_like_native() {
    check(Some("1"));
}

#[test]
fn one_orients_top_heavy_mesh_with_antiparallel_rotation_like_native() {
    check_with_vertices(
        Some("1"),
        Some(
            r#"
<vertex x="0" y="0" z="-3"/><vertex x="2" y="19" z="-3"/>
<vertex x="27" y="0" z="-3"/><vertex x="3" y="2" z="-5"/>
"#,
        ),
    );
}

#[test]
fn other_integer_retains_automatic_saved_project_eligibility_like_native() {
    check(Some("2"));
    check(Some("-1"));
}

fn check(orient: Option<&str>) {
    check_with_vertices(orient, None);
}

fn check_with_vertices(orient: Option<&str>, vertices: Option<&str>) {
    check_project(
        orient,
        &asymmetric_project(vertices),
        None,
        "asymmetric",
        true,
    );
}

fn check_project(
    orient: Option<&str>,
    project: &[u8],
    plate: Option<u32>,
    label: &str,
    expected_change: bool,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let actions = if orient == Some("1") {
        vec![Some("0"), orient]
    } else {
        vec![orient]
    };
    let mut streams = Vec::new();
    for action in actions {
        let root = tempfile::Builder::new()
            .prefix(&format!("orient-{label}-{}-", action.unwrap_or("default")))
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let input = root.join(format!("{label}.3mf"));
        fs::write(&input, project).unwrap();
        let output = root.join("native");
        fs::create_dir(&output).unwrap();
        let mut args = vec!["--debug".into(), "2".into()];
        if let Some(value) = action {
            args.extend(["--orient".into(), value.into()]);
        }
        args.extend([
            "--slice".into(),
            plate.unwrap_or(0).to_string(),
            "--outputdir".into(),
            output.to_str().unwrap().into(),
            input.to_str().unwrap().into(),
        ]);
        stages::run_live(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
        let native = fs::read(output.join("plate_1.gcode")).unwrap();
        fs::write(root.join("native.gcode"), &native).unwrap();
        let ares_path = root.join("ares.gcode");
        let mut cli = cli_command();
        cli.args(["slice", "-o"]).arg(&ares_path);
        if let Some(value) = action {
            cli.args(["--orient", value]);
        }
        if let Some(plate) = plate {
            cli.args(["--plate", &plate.to_string()]);
        }
        let actual = cli.arg(&input).output().unwrap();
        fs::write(root.join("ares.stdout"), &actual.stdout).unwrap();
        fs::write(root.join("ares.stderr"), &actual.stderr).unwrap();
        fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "orient": action, "input_sha256": crate::runner::native_api_cli::sha(project),
            "fixture": label, "plate": plate, "expected_output_change": expected_change,
            "native_sha256": crate::runner::native_api_cli::sha(&native), "ares_exit": actual.status.code(),
            "comparison": "complete ordered bytes; generated-by only",
            "upstream": "Orient.cpp AutoOrienter; OrcaSlicer.cpp:4973-5003,5271-5288"
        })).unwrap()).unwrap();
        assert!(
            actual.status.success(),
            "Ares CLI failed; artifacts {root:?}"
        );
        let ares = fs::read(ares_path).unwrap();
        let native =
            golden::normalize_one_generator_line(&native, golden::GeneratorKind::Orca).unwrap();
        let ares =
            golden::normalize_one_generator_line(&ares, golden::GeneratorKind::Ares).unwrap();
        assert!(
            golden::first_difference(&native, &ares).is_none(),
            "complete output mismatch: {:?}; artifacts {root:?}",
            golden::first_difference(&native, &ares).map(|difference| difference.to_string())
        );
        streams.push(native);
    }
    if orient == Some("1") {
        assert_eq!(
            streams[0] != streams[1],
            expected_change,
            "unexpected Native action outcome for {label}"
        );
    }
}
