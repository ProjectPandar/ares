use serde_json::json;

fn settings(angle: &str, skirt_height: &str) -> Vec<(&'static str, serde_json::Value)> {
    vec![
        ("belt_slice_rotation_angle", json!(angle)),
        ("spiral_mode", json!("1")),
        ("wall_loops", json!("1")),
        ("bottom_shell_layers", json!("2")),
        ("top_shell_layers", json!("0")),
        ("sparse_infill_density", json!("0%")),
        ("skirt_loops", json!("0")),
        ("skirt_height", json!(skirt_height)),
        ("enable_support", json!("0")),
    ]
}

#[test]
fn belt_spiral_x90_matches_native() {
    let edits = settings("90", "0");
    super::compare_output("spiral-x90", true, false, &edits, |_| {});
}

#[test]
fn belt_spiral_x45_matches_native() {
    let edits = settings("45", "0");
    super::compare_output("spiral-x45", true, false, &edits, |_| {});
}

#[test]
fn belt_spiral_skirt_height_delays_spiralization() {
    let zero = settings("90", "0");
    super::compare_output("spiral-skirt-height-zero", true, false, &zero, |_| {});
    let four = settings("90", "4");
    super::compare_output("spiral-skirt-height-four", true, false, &four, |_| {});
}

#[test]
fn belt_spiral_identity_remap_matches_native() {
    let mut edits = settings("90", "0");
    edits.extend([
        ("gcode_remap_x", json!("pos_x")),
        ("gcode_remap_y", json!("pos_y")),
        ("gcode_remap_z", json!("pos_z")),
    ]);
    super::compare_output("spiral-identity-remap", true, false, &edits, |_| {});
}

#[test]
fn flat_bed_spiral_output_matches_native() {
    let edits = settings("90", "0");
    super::compare_output("spiral-flat-bed", false, false, &edits, |_| {});
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_belt_spiral_with_active_brim() {
    use crate::{OrcaRunner, native_wall_clock::result_comparison_bytes, runner::stages};
    use serde_json::Value;
    use std::{fs, path::Path};

    const DETAIL: &str =
        "Brim is not compatible with spiral vase mode on a belt printer. Disable one of them.";
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt spiral brim rejection requires Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir().join("spiral-active-brim");
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    let mut edits = settings("45", "0");
    edits.extend([
        ("brim_type", json!("outer_only")),
        ("brim_width", json!("2")),
    ]);
    fs::write(
        work.join("input.3mf"),
        super::project(true, false, &edits, false),
    )
    .unwrap();
    let native = stages::run(
        runner.bin(),
        super::ARGS.map(str::to_owned),
        stages::Stage::Slice,
        &work,
    )
    .unwrap_err()
    .captured
    .unwrap();
    assert_eq!(native.code, Some(205), "{work:?}");
    assert!(
        String::from_utf8_lossy(&native.stderr)
            .lines()
            .any(|line| line == DETAIL),
        "{work:?}"
    );
    let report = |folder: &Path| {
        assert!(!folder.join("plate_1.gcode").exists(), "{folder:?}");
        let bytes = fs::read(folder.join("result.json")).unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["return_code"], -51, "{folder:?}");
        result_comparison_bytes(&bytes)
    };
    let native_report = report(&out);
    fs::rename(&out, work.join("native")).unwrap();
    fs::create_dir(&out).unwrap();
    let actual = super::run_ares(&work);
    assert_eq!(actual.status.code(), native.code, "{work:?}");
    assert!(
        String::from_utf8_lossy(&actual.stderr)
            .lines()
            .any(|line| line == DETAIL),
        "{work:?}"
    );
    assert_eq!(report(&out), native_report, "{work:?}: validation report");
}
