//! GCode.cpp:9222-9245: `_extrude` travels to the first point only on `!m_last_pos_defined`, a
//! different last position, `m_need_change_layer_lift_z` or a slope Z travel. The writer's
//! position-clear flag, cleared by the timelapse (GCode.cpp:6481), only shapes the move inside
//! GCodeWriter (GCodeWriter.cpp:1006, :1147).
use crate::runner::{application, native_stream_case};
use std::fs;

const CASE: (&str, &[&str]) = (
    "first-point-travel",
    &[
        "GCode.cpp:9222-9245",
        "GCode.cpp:6466-6513",
        "GCode.cpp:8453",
        "GCodeWriter.cpp:1006,1147",
    ],
);

/// Every window from an `M625` to the next `; FEATURE:` line.
fn label_end_windows(text: &str) -> Vec<Vec<&str>> {
    let mut windows = Vec::new();
    let mut open: Option<Vec<&str>> = None;
    for line in text.lines() {
        if line == "M625" {
            open = Some(Vec::new());
        } else if let Some(window) = open.as_mut() {
            if line.starts_with("; FEATURE:") {
                windows.push(open.take().unwrap());
            } else {
                window.push(line);
            }
        }
    }
    windows
}

fn bare_z_moves(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            line.strip_prefix("G1 Z").is_some_and(|z| {
                !z.is_empty() && z.bytes().all(|b| b.is_ascii_digit() || b == b'.')
            })
        })
        .collect()
}

/// Native, API and CLI output of `input`; API and CLI must agree byte for byte.
fn slice(label: &str, input: &[u8], sha: &str) -> (String, String) {
    let (reference, _) = native_stream_case::qualify(CASE, label, input, sha, false);
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let path = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success());
    assert_eq!(api, fs::read(&path).unwrap(), "raw API/CLI equality");
    (reference.native, String::from_utf8(api).unwrap())
}

#[test]
fn timelapse_label_end_writes_no_travel_to_the_current_point() {
    let (native, ares) = slice(
        "p1s-bylayer-control",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/unknown-key-config-block/p1s-bylayer-control.3mf"
        )),
        "2ff89639d4f6c6898393690e3236b6a1fff7edc116c3ba77c3eb8f98de5ff6ca",
    );
    let windows = label_end_windows(&native);
    assert!(
        native.contains("\nM971 "),
        "Native writes the inline timelapse"
    );
    assert!(
        windows
            .iter()
            .any(|window| window.iter().all(|line| !line.starts_with("G1 "))),
        "Native has a label end followed by its feature with no travel"
    );
    assert_eq!(label_end_windows(&ares), windows);
}

fn check_spiral_z_restatements(label: &str, input: &[u8], sha: &str) {
    let (native, ares) = slice(label, input, sha);
    let moves = bare_z_moves(&native);
    assert!(!moves.is_empty(), "Native re-states the layer Z");
    assert_eq!(bare_z_moves(&ares), moves);
}

#[test]
fn spiral_vase_single_object_keeps_layer_z_restatements() {
    check_spiral_z_restatements(
        "spiral-single-object",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-vase-admission/single-object-by-layer.3mf"
        )),
        "a8d0ed5f032f22a3b8968c9c648f621b913323abebaa417077f359b112b0ae38",
    );
}

#[test]
fn spiral_vase_by_object_keeps_layer_z_restatements() {
    check_spiral_z_restatements(
        "spiral-by-object-apart",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"
        )),
        "2dbf797da35adf51f085c397cfa959364fd7411a73ef52ec687d04dd480e7342",
    );
}
