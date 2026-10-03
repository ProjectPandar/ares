use super::{CaseInputs, OrcaRunner, ParityCase, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::{Map, json};

#[test]
fn orca_parity_arachne_zero_threshold_reverses_real_moves_and_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let ordinary = capture(&runner, "0");
    let reversed = capture(&runner, "1");
    let moves = |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .unwrap()
            .split(";LAYER_CHANGE")
            .nth(2)
            .unwrap()
            .lines()
            .filter(|line| {
                line.starts_with("G1 ")
                    && line.contains(" E")
                    && (line.contains(" X") || line.contains(" Y"))
            })
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let ordinary_moves = moves(&ordinary.reference);
    let reversed_moves = moves(&reversed.reference);
    assert!(!ordinary_moves.is_empty());
    assert!(!reversed_moves.is_empty());
    assert_ne!(
        ordinary_moves, reversed_moves,
        "native reversal must change extrusion movement, not just configuration text"
    );
    for case in [ordinary, reversed] {
        let outcome = parity::compare_case(&case);
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}

fn capture(runner: &OrcaRunner, reverse: &str) -> ParityCase {
    capture_settings(runner, (reverse, "0%", "0", "1"), "cube10.stl")
}

#[test]
fn orca_parity_arachne_internal_only_hole_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    for internal_only in ["0", "1"] {
        let case = capture_settings(&runner, ("1", "0%", internal_only, "1"), "arachne/hole.stl");
        let outcome = parity::compare_case(&case);
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}

#[test]
fn orca_parity_arachne_nonzero_reverse_threshold_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    for threshold in ["0.1", "50%"] {
        let case = capture_settings(&runner, ("1", threshold, "0", "1"), "overhang.stl");
        let outcome = parity::compare_case(&case);
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}

#[test]
fn orca_parity_arachne_reverse_without_detection_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = capture_settings(&runner, ("1", "50%", "1", "0"), "arachne/hole.stl");
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn orca_parity_arachne_hole_without_reversal_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = capture_settings(&runner, ("0", "50%", "0", "1"), "arachne/hole.stl");
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn orca_parity_arachne_overhang_without_reversal_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = capture_settings(&runner, ("0", "0.1", "0", "1"), "overhang.stl");
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn orca_parity_arachne_overhang_without_detection_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = capture_settings(&runner, ("0", "0.1", "0", "0"), "overhang.stl");
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn orca_parity_arachne_supported_reverse_boundaries_preserve_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    for (threshold, internal, detect) in [
        ("0%", "1", "1"),
        ("0.1", "0", "1"),
        ("50%", "0", "1"),
        ("20", "0", "1"),
        ("50%", "1", "0"),
    ] {
        let case = capture_settings(&runner, ("1", threshold, internal, detect), "cube10.stl");
        let outcome = parity::compare_case(&case);
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}

fn capture_settings(
    runner: &OrcaRunner,
    (reverse, threshold, internal_only, detect): (&str, &str, &str, &str),
    model: &str,
) -> ParityCase {
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("wall_generator", "arachne"),
        ("wall_loops", "3"),
        ("overhang_reverse", reverse),
        ("overhang_reverse_threshold", threshold),
        ("overhang_reverse_internal_only", internal_only),
        ("detect_overhang_wall", detect),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let label =
        format!("arachne-overhang-reverse/{model}/{reverse}/{threshold}/{internal_only}/{detect}");
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &Map::new(),
            &repo_root().join("tests/parity").join(model),
        )
        .unwrap();
    for (key, value) in settings {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    runner.slice_case(&exported).unwrap()
}
