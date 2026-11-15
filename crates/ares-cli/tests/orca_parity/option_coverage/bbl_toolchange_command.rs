//! GCodeWriter.cpp:793-830: the tool-selection prefix has no BBL branch, and a BBL printer writes
//! `M1020 S<id> H<nozzle>` only when the change template selects no tool (GCode.cpp:11237-11247),
//! with the nozzle of nozzle_id_for_gcode_placeholder (GCode.cpp:241-247).
use crate::runner::{application, native_stream_case};
use std::fs;

const CASE: (&str, &[&str]) = (
    "bbl-toolchange-command",
    &[
        "GCodeWriter.cpp:793-830",
        "GCode.cpp:241-247",
        "GCode.cpp:11237-11247",
        "CoolingBuffer.cpp:40",
    ],
);

fn selections(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            line.starts_with("M1020 ")
                || line
                    .strip_prefix('T')
                    .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        })
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, premise: fn(&[&str])) {
    let (reference, _) = native_stream_case::qualify(CASE, label, input, sha, false);
    let native = selections(&reference.native);
    premise(&native);
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
    let cli = fs::read(&path).unwrap();
    assert_eq!(api, cli, "raw API/CLI equality");
    let api = std::str::from_utf8(&api).unwrap();
    assert_eq!(selections(api), native, "API tool selection lines");
    assert_eq!(
        selections(std::str::from_utf8(&cli).unwrap()),
        native,
        "CLI tool selection lines"
    );
}

#[test]
fn x1c_template_tool_selection_writes_no_extra_m1020() {
    check(
        "x1c-template-selects-tool",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bbl-layer-object-labels/two-object.3mf"
        )),
        "e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28",
        |native| {
            assert!(native.iter().all(|line| !line.starts_with("M1020")));
            assert!(native.contains(&"T0") && native.contains(&"T1"));
        },
    );
}

#[test]
fn x1c_template_without_tool_selection_writes_m1020_with_nozzle() {
    check(
        "x1c-template-without-selection",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m1020-filament-change/towerless-change.3mf"
        )),
        "8a2a9b965621cff0a3a274bda6db68cf97a02f460c8ea9fe375a14c1dba099a7",
        |native| {
            assert!(native.contains(&"M1020 S1 H0") && native.contains(&"M1020 S0 H0"));
            assert!(!native.contains(&"T0") && !native.contains(&"T1"));
        },
    );
}

#[test]
fn ender3_multi_tool_keeps_t_selection() {
    check(
        "ender3-multi-tool",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bbl-layer-object-labels/non-bbl.3mf"
        )),
        "33980328409429587aa8d6b2d97be1d5755c010917113018f2c8fb2bf2e6ffe5",
        |native| {
            assert!(native.iter().all(|line| !line.starts_with("M1020")));
            assert!(native.contains(&"T0") && native.contains(&"T1"));
        },
    );
}
