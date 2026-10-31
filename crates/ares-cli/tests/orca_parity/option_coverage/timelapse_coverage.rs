//! GCode.cpp:4645-4675,5145-5202: real timelapse template branch witnesses.
use super::*;
use serde_json::json;

#[path = "timelapse_coverage/input.rs"]
pub(super) mod input;

#[test]
fn timelapse_type_conditional_template_and_empty_controls() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let evidence = root.join("timelapse-type-bounded.json");
    let mut proofs = Vec::new();
    for active in [true, false] {
        for mode in ["0", "1"] {
            let label = format!("timelapse/template/{active}/{mode}");
            let exported = input::export(&runner, &json!(mode), active, &label).unwrap();
            let sliced = runner.slice_case(&exported).unwrap();
            let native = std::str::from_utf8(&sliced.reference).unwrap();
            let markers = branch_lines(native, mode, active);
            let outcome = parity::compare_case(&sliced);
            proofs.push(json!({"label":label,"mode":mode,"template_active":active,
                "marker_lines":markers,"full_output_status":outcome.status,
                "detail":outcome.detail,"artifacts":outcome.artifacts}));
            parity::artifacts::overwrite(&evidence, &serde_json::to_vec_pretty(&proofs).unwrap())
                .unwrap();
            assert_eq!(
                outcome.status, "PASS",
                "{label}: {}; {root:?}",
                outcome.detail
            );
        }
    }
}

fn branch_lines<'a>(native: &'a str, mode: &str, active: bool) -> Vec<(usize, &'a str)> {
    let expected = if mode == "0" {
        ";W6_TL_TRAD "
    } else {
        ";W6_TL_SMOOTH "
    };
    let other = if mode == "0" {
        ";W6_TL_SMOOTH "
    } else {
        ";W6_TL_TRAD "
    };
    let markers = native
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with(expected))
        .collect::<Vec<_>>();
    assert_eq!(
        !markers.is_empty(),
        active,
        "timelapse mode={mode}, active={active}"
    );
    assert!(!native.lines().any(|line| line.starts_with(other)));
    for (_, line) in &markers {
        assert!(
            !line.contains('{') && !line.contains('}'),
            "unexpanded {line}"
        );
        let fields = line.split_whitespace().collect::<Vec<_>>();
        assert_eq!(fields.len(), 4, "{line}");
        for (field, prefix) in fields[1..].iter().zip(["layer=", "z=", "max="]) {
            assert!(
                field
                    .strip_prefix(prefix)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    .is_finite(),
                "{line}"
            );
        }
    }
    markers
}

// GCodeProcessor.cpp:1922-1948 / OrcaSlicer.cpp:6221-6227: tower/skirt
// outside the original preset bed is a rejection, never success coverage.
#[test]
fn timelapse_original_smooth_sweep_retains_minus102_control() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = input::rejected_smooth(&runner).unwrap();
    let settings = runner::application::settings(&exported.project).unwrap();
    for (key, expected) in [
        ("timelapse_type", json!("1")),
        ("enable_prime_tower", json!("1")),
        ("time_lapse_gcode", json!("")),
        ("wipe_tower_y", json!(["220"])),
    ] {
        assert_eq!(settings[key], expected, "original rejection context {key}");
    }
    let root = parity::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("timelapse-rejection-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    std::fs::write(&path, &exported.project).unwrap();
    let failure = runner::stages::run(
        runner.bin(),
        [
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            directory.to_str().unwrap().into(),
            path.to_str().unwrap().into(),
        ],
        runner::stages::Stage::Slice,
        &directory,
    )
    .unwrap_err();
    assert_eq!(
        failure.captured.as_ref().unwrap().code,
        Some(if cfg!(unix) { 154 } else { -102 }),
        "{failure}"
    );
    let result: Value =
        serde_json::from_slice(&std::fs::read(directory.join("result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], -102);
    assert_eq!(
        result["error_string"],
        "Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer."
    );
    assert!(directory.join("plate_1.gcode").exists());
}
