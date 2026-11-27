use serde_json::json;

#[test]
fn echoes_combined_skirt_settings_without_belt_skirt_extrusion() {
    compare("skirt-combined", "combined", "4", false);
}

#[test]
fn echoes_per_object_skirt_settings_without_belt_skirt_extrusion() {
    compare("skirt-per-object", "perobject", "4", false);
}

#[test]
fn accepts_belt_skirt_loops_with_zero_skirt_height() {
    compare("skirt-zero-height", "combined", "0", false);
}

#[test]
fn per_object_skirt_does_not_displace_the_belt_brim() {
    compare("skirt-with-brim", "perobject", "4", true);
}

fn compare(name: &str, kind: &str, height: &str, brim: bool) {
    let mut settings = vec![
        ("skirt_loops", json!("3")),
        ("skirt_height", json!(height)),
        ("skirt_type", json!(kind)),
        ("draft_shield", json!("disabled")),
    ];
    if brim {
        settings.extend([
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("outer_only")),
            ("brim_width", json!("2")),
        ]);
    }
    super::compare_output(name, true, false, &settings, |gcode| {
        let gcode = std::str::from_utf8(gcode).unwrap();
        assert!(!gcode.lines().any(|line| line == ";TYPE:Skirt"));
        assert_eq!(gcode.lines().any(|line| line == ";TYPE:Brim"), brim);
        for (key, value) in [
            ("skirt_loops", "3"),
            ("skirt_height", height),
            ("skirt_type", kind),
        ] {
            let entry = format!("; {key} = {value}");
            assert!(gcode.lines().any(|line| line == entry), "missing {entry}");
        }
    });
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_a_belt_draft_shield_even_without_skirt_loops() {
    rejects_draft_shield("0");
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_a_belt_draft_shield_with_skirt_loops() {
    rejects_draft_shield("3");
}

#[cfg(target_os = "linux")]
fn rejects_draft_shield(loops: &str) {
    use crate::{OrcaRunner, native_wall_clock::result_comparison_bytes, runner::stages};
    use serde_json::Value;
    use std::{fs, path::Path};

    const DETAIL: &str = "Draft shield is not compatible with belt printer mode.";
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt draft shield requires Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir();
    let out = work.join("out");
    fs::create_dir(&out).unwrap();
    let settings = [
        ("draft_shield", json!("enabled")),
        ("skirt_loops", json!(loops)),
        ("skirt_height", json!(if loops == "0" { "0" } else { "4" })),
    ];
    fs::write(
        work.join("input.3mf"),
        super::project(true, false, &settings, false),
    )
    .unwrap();
    let native = stages::run(
        runner.bin(),
        super::ARGS.map(str::to_owned),
        stages::Stage::Slice,
        work,
    )
    .unwrap_err()
    .captured
    .unwrap();
    assert_eq!(native.code, Some(205), "{work:?}");
    let assert_diagnostic = |stderr: &[u8]| {
        assert!(
            String::from_utf8_lossy(stderr)
                .lines()
                .any(|line| line == DETAIL),
            "{work:?}"
        );
    };
    assert_diagnostic(&native.stderr);
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
    let actual = super::run_ares(work);
    assert_eq!(actual.status.code(), native.code, "{work:?}");
    assert_diagnostic(&actual.stderr);
    assert!(
        report(&out) == native_report,
        "validation report differs in {work:?}"
    );
}
