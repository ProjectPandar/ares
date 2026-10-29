//! Native LibVGCodeWrapper conversion + libvgcode `ExtrusionRoles` + extracted
//! `ViewerImpl.cpp:1002-1018` role/travel time oracle.
//! Failure inventory: f64 or reordered sums; roles by first appearance instead
//! of enum order; non-Extrude vertices (wipe, retract, seam) counted into a
//! role; wipe time counted as travel; absent role not 0; Stealth bits instead
//! of Normal; state kept from a previous load.
//! Public seams: `ViewerImpl::load`, then the `ViewerImpl.hpp:141-147` getters.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use ares_vgcode::{GCodeExtrusionRole, GCodeInputData, MoveType, PathVertex, ViewerImpl};

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

const TEST: &str = "native_role_and_travel_time_bits";
const FIXTURES: [&str; 4] = [
    "role-times/roles.gcode",
    "actual-speed-profile/profile.gcode",
    "marker-time/path.gcode",
    "color-change/normal.gcode",
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(gcode: &str) -> Option<PathBuf> {
    native_environment::binary("ARES_ROLE_TIMES_BIN")?;
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
        .prefix("role-times-live-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("live role-times artifacts for {gcode}: {directory:?}");
    let output = Command::new("node")
        .arg(repo().join("tools/chain-probe/role-times-fixtures.mjs"))
        .arg(&directory)
        .arg(repo().join("tests/parity").join(gcode))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "live role-times oracle failed: {} (artifacts {directory:?})",
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

fn vertices(root: &Path) -> Vec<PathVertex> {
    rows(root, "vertices.tsv")
        .iter()
        .enumerate()
        .map(|(id, values)| {
            assert_eq!(values[0] as usize, id);
            PathVertex {
                move_type: [
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
                ][values[1] as usize],
                role: GCodeExtrusionRole::ALL[values[2] as usize],
                times: [values[3], values[4]].map(f32::from_bits),
                ..Default::default()
            }
        })
        .collect()
}

#[test]
fn required_missing_native_cannot_pass() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_ROLE_TIMES_BIN")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ARES_REQUIRE_ORCA=1: missing ARES_ROLE_TIMES_BIN"),
        "required failure did not identify the missing Native producer",
    );
}

#[test]
fn optional_missing_native_reports_skipped() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env_remove("ARES_REQUIRE_ORCA")
        .env_remove("ARES_ROLE_TIMES_BIN")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SKIPPED:"));
}

#[test]
fn native_role_and_travel_time_bits() {
    // One viewer across fixtures: a load must not keep the previous totals.
    let mut viewer = ViewerImpl::default();
    for gcode in FIXTURES {
        let Some(root) = fixture(gcode) else {
            return;
        };
        let vertices = vertices(&root);
        let roles = rows(&root, "roles.tsv");
        let totals = rows(&root, "totals.tsv");
        if gcode == FIXTURES[0] {
            // The fixture must reach every rule: out-of-enum-order first
            // appearance, wipe and non-extrusion options, travel time > 0.
            assert!(vertices.iter().any(|v| v.move_type == MoveType::Wipe));
            assert!(vertices.iter().any(|v| v.move_type == MoveType::Retract));
            let first_seen: Vec<_> = vertices
                .iter()
                .filter(|v| v.move_type == MoveType::Extrude)
                .map(|v| v.role as u32)
                .collect();
            assert!(first_seen.windows(2).any(|w| w[0] > w[1]));
            assert!(roles.len() >= 6);
            assert_ne!(totals[0][0], 0);
        }
        viewer.load(GCodeInputData {
            vertices,
            ..Default::default()
        });
        let expected: Vec<_> = roles
            .iter()
            .map(|r| GCodeExtrusionRole::ALL[r[0] as usize])
            .collect();
        assert_eq!(
            viewer.get_extrusion_roles().collect::<Vec<_>>(),
            expected,
            "{gcode} roles"
        );
        for role in GCodeExtrusionRole::ALL {
            let bits = roles
                .iter()
                .find(|r| r[0] == role as u32)
                .map_or(0, |r| r[1]);
            assert_eq!(
                viewer.get_extrusion_role_estimated_time(role).to_bits(),
                bits,
                "{gcode} {role:?} Normal time"
            );
        }
        // totals.tsv row 0 is travels; column 0 the Normal mode.
        assert_eq!(
            viewer.get_travels_estimated_time().to_bits(),
            totals[0][0],
            "{gcode} travels Normal time"
        );
    }
}
