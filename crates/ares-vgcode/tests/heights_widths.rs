//! Native LibVGCodeWrapper conversion + extracted libvgcode
//! `extract_pos_and_or_hwa` height/width branch (`ViewerImpl.cpp:954-968`) and
//! radius setters (`ViewerImpl.cpp:1648-1658`) oracle.
//! Failure inventory: radius doubled or halved instead of the full shader size;
//! wipes taking the travels radius; options other than travel/wipe taking a
//! radius instead of their own height/width; clamp missing or with the wrong
//! limits; defaults other than 0.1; sizes drifting through f64.
//! Public seams: `ViewerImpl::load`, then `heights_widths` and the radius
//! getters/setters (`ViewerImpl.hpp:169-172`).

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use ares_vgcode::{GCodeInputData, MoveType, PathVertex, ViewerImpl};

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

const TEST: &str = "native_segment_heights_widths_bits";
const FIXTURES: [&str; 3] = [
    "role-times/roles.gcode",
    "marker-time/path.gcode",
    "color-change/normal.gcode",
];
const MOVE_TYPES: [MoveType; 11] = [
    MoveType::Noop,
    MoveType::Retract,
    MoveType::Unretract,
    MoveType::Seam,
    MoveType::ToolChange,
    MoveType::ColorChange,
    MoveType::PausePrint,
    MoveType::CustomGCode,
    MoveType::Travel,
    MoveType::Wipe,
    MoveType::Extrude,
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(gcode: &str) -> Option<PathBuf> {
    native_environment::binary("ARES_SEGMENT_HW_BIN")?;
    match Command::new("node").arg("--version").output() {
        Ok(output) => assert!(output.status.success(), "node --version failed"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            assert!(
                !native_environment::required(),
                "ARES_REQUIRE_ORCA=1: missing node; Native comparison not executed"
            );
            native_environment::skipped("node is unavailable");
            return None;
        }
        Err(error) => panic!("cannot launch node: {error}"),
    }
    let directory = tempfile::Builder::new()
        .prefix("segment-hw-live-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("live segment-hw artifacts for {gcode}: {directory:?}");
    let output = Command::new("node")
        .arg(repo().join("tools/chain-probe/segment-hw-fixtures.mjs"))
        .arg(&directory)
        .arg(repo().join("tests/parity").join(gcode))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "live segment-hw oracle failed: {} (artifacts {directory:?})",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(directory)
}

fn rows(root: &Path, name: &str) -> Vec<Vec<u32>> {
    std::fs::read_to_string(root.join(name))
        .unwrap()
        .lines()
        .map(|line| line.split('\t').map(|v| v.parse().unwrap()).collect())
        .collect()
}

#[test]
fn required_missing_native_cannot_pass() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_SEGMENT_HW_BIN")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ARES_REQUIRE_ORCA=1: missing ARES_SEGMENT_HW_BIN"),
        "required failure did not identify the missing Native producer",
    );
}

#[test]
fn optional_missing_native_reports_skipped() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env_remove("ARES_REQUIRE_ORCA")
        .env_remove("ARES_SEGMENT_HW_BIN")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SKIPPED:"));
}

#[test]
fn native_segment_heights_widths_bits() {
    for gcode in FIXTURES {
        let Some(root) = fixture(gcode) else {
            return;
        };
        let table = rows(&root, "heights_widths.tsv");
        let vertices: Vec<_> = table
            .iter()
            .map(|r| PathVertex {
                move_type: MOVE_TYPES[r[0] as usize],
                height: f32::from_bits(r[1]),
                width: f32::from_bits(r[2]),
                ..Default::default()
            })
            .collect();
        if gcode == FIXTURES[0] {
            // The fixture must reach every branch: travel, wipe, extrusion and
            // an option with its own size.
            for kind in [
                MoveType::Travel,
                MoveType::Wipe,
                MoveType::Extrude,
                MoveType::Retract,
            ] {
                assert!(vertices.iter().any(|v| v.move_type == kind), "{kind:?}");
            }
        }
        let mut viewer = ViewerImpl::default();
        viewer.load(GCodeInputData {
            vertices,
            ..Default::default()
        });
        assert_eq!(viewer.get_travels_radius().to_bits(), 0.1f32.to_bits());
        assert_eq!(viewer.get_wipes_radius().to_bits(), 0.1f32.to_bits());
        let actual: Vec<_> = viewer
            .heights_widths()
            .iter()
            .map(|hw| hw.map(f32::to_bits))
            .collect();
        let expected: Vec<_> = table.iter().map(|r| [r[3], r[4]]).collect();
        assert_eq!(actual, expected, "{gcode} heights/widths bits");

        for clamp in rows(&root, "radius_clamps.tsv") {
            viewer.set_travels_radius(f32::from_bits(clamp[0]));
            viewer.set_wipes_radius(f32::from_bits(clamp[0]));
            assert_eq!(
                [viewer.get_travels_radius(), viewer.get_wipes_radius()].map(f32::to_bits),
                [clamp[1], clamp[2]],
                "radius {} clamp",
                f32::from_bits(clamp[0])
            );
        }
        // The last probe (2.0) clamps to 1.0; the next sizes use it.
        for (hw, r) in viewer.heights_widths().iter().zip(&table) {
            if matches!(MOVE_TYPES[r[0] as usize], MoveType::Travel | MoveType::Wipe) {
                assert_eq!(
                    hw.map(f32::to_bits),
                    [1.0f32.to_bits(); 2],
                    "{gcode} resized option"
                );
            }
        }
    }
}
