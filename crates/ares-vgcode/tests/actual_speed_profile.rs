//! Native LibVGCodeWrapper conversion + extracted `GCodeViewer.cpp:1682-1752`
//! actual speed profile oracle.
//! Failure inventory: group start/end off by one or the last buffer vertex kept;
//! seam `--end_id` on the wrong vertex; f64 lengths; first point skipped;
//! `roundf` vs ties-to-even compression, duplicate guard or final point;
//! `internal` from another time mode; y range/levels from the wrong range or
//! travel/wipe visibility; trigger against the full range; stale data cleared.
//! Public seams: `SequentialView::update_actual_speed` after `ViewerImpl::colors`.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use ares_vgcode::{
    ActualSpeedProfile, GCodeExtrusionRole, GCodeInputData, MoveType, OptionType, PathVertex,
    SequentialView, ViewerImpl,
};

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

const TEST: &str = "native_profile_bits_for_every_marker_vertex";

fn fixture() -> Option<PathBuf> {
    native_environment::binary("ARES_ACTUAL_SPEED_PROFILE_BIN")?;
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
        .prefix("actual-speed-profile-live-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("live actual-speed-profile artifacts: {directory:?}");
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("node")
        .arg(repo.join("tools/chain-probe/actual-speed-profile-fixtures.mjs"))
        .arg(&directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "live actual-speed-profile oracle failed: {} (artifacts {directory:?})",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(directory)
}

fn rows(root: &Path, name: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(root.join(name))
        .unwrap()
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect()
}

fn vertices(root: &Path) -> Vec<PathVertex> {
    rows(root, "vertices.tsv")
        .iter()
        .enumerate()
        .map(|(id, values)| {
            let uint = |i: usize| values[i].parse::<u32>().unwrap();
            let float = |i: usize| f32::from_bits(uint(i));
            assert_eq!(uint(0) as usize, id);
            PathVertex {
                gcode_id: uint(1),
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
                ][uint(2) as usize],
                role: GCodeExtrusionRole::ALL[uint(3) as usize],
                layer_id: uint(4),
                position: [5, 6, 7].map(float),
                times: [8, 9].map(float),
                actual_feedrate: float(10),
                ..Default::default()
            }
        })
        .collect()
}

/// Native profiles per vertex id for one visibility (`None`: no update),
/// flattened as `[y_min, y_max, levels, (value, r, g, b, a)*, items, (pos, speed, internal)*]`.
fn profiles(root: &Path, visibility: &str) -> Vec<Option<Vec<u32>>> {
    rows(root, "profiles.tsv")
        .into_iter()
        .filter(|values| values[0] == visibility)
        .enumerate()
        .map(|(id, values)| {
            assert_eq!(values[1].parse::<usize>().unwrap(), id);
            (values[2] == "1").then(|| values[3..].iter().map(|v| v.parse().unwrap()).collect())
        })
        .collect()
}

fn flat(profile: &ActualSpeedProfile) -> Vec<u32> {
    let mut bits = vec![
        profile.y_range[0].to_bits(),
        profile.y_range[1].to_bits(),
        profile.levels.len() as u32,
    ];
    for (value, color) in &profile.levels {
        bits.push(value.to_bits());
        bits.extend(color.map(f32::to_bits));
    }
    bits.push(profile.data.len() as u32);
    for item in &profile.data {
        bits.extend([
            item.pos.to_bits(),
            item.speed.to_bits(),
            u32::from(item.internal),
        ]);
    }
    bits
}

#[test]
fn required_missing_native_cannot_pass() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_ACTUAL_SPEED_PROFILE_BIN")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ARES_REQUIRE_ORCA=1: missing ARES_ACTUAL_SPEED_PROFILE_BIN"),
        "required failure did not identify the missing Native producer",
    );
}

#[test]
fn optional_missing_native_reports_skipped() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env_remove("ARES_REQUIRE_ORCA")
        .env_remove("ARES_ACTUAL_SPEED_PROFILE_BIN")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SKIPPED:"));
}

#[test]
fn native_profile_bits_for_every_marker_vertex() {
    let Some(root) = fixture() else {
        return;
    };
    let vertices = vertices(&root);
    let hidden = profiles(&root, "hidden");
    let shown = profiles(&root, "shown");
    // The fixture must reach every rule: seam end, interpolated points,
    // compression, ineligible vertices and the travel/wipe range.
    assert!(vertices.iter().any(|v| v.move_type == MoveType::Seam));
    let updated: Vec<_> = hidden.iter().flatten().collect();
    assert!(updated.len() < hidden.len());
    assert!(updated.iter().any(|p| {
        let levels = p[2] as usize;
        let items = &p[4 + 5 * levels..];
        items.len() >= 9 && items.chunks(3).any(|item| item[2] == 1)
    }));
    assert_ne!(hidden, shown);

    let mut viewer = ViewerImpl::default();
    viewer.load(GCodeInputData {
        vertices,
        ..Default::default()
    });
    for (visibility, native) in [("hidden", hidden), ("shown", shown)] {
        if visibility == "shown" {
            viewer.toggle_option_visibility(OptionType::Travels);
            viewer.toggle_option_visibility(OptionType::Wipes);
        }
        viewer.colors();
        let [first, last] = viewer.get_view_enabled_range();
        let mut view = SequentialView::default();
        let mut expected = flat(view.actual_speed());
        // Reverse then forward seeks: a skipped update keeps the previous profile.
        for id in (first..=last).rev().chain(first..=last) {
            viewer.set_view_visible_range(first, id);
            view.update_actual_speed(&viewer);
            if id != last
                && let Some(profile) = &native[id]
            {
                expected.clone_from(profile);
            }
            assert_eq!(
                flat(view.actual_speed()),
                expected,
                "{visibility} vertex {id}"
            );
        }
    }
}
