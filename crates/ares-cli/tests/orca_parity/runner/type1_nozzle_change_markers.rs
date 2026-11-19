use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};

use crate::{
    OrcaRunner,
    runner::{application, native_api_cli::NativeCase},
};

const START: &str = "; NOZZLE_CHANGE_START OF0 NF1 ON1 NN0";
const END: &str = "; NOZZLE_CHANGE_END OF0 NF1 ON1 NN0";

fn markers(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| line.starts_with("; NOZZLE_CHANGE_"))
        .collect()
}

fn check_setup(gcode: &str, (map, tower): (&str, bool)) {
    assert!(gcode.lines().any(|line| line == map), "physical nozzle map");
    assert!(
        gcode
            .lines()
            .any(|line| line == "; enable_arc_fitting = enabled")
    );
    assert!(
        gcode.lines().any(|line| line.trim_end() == "T1"),
        "logical filament change"
    );
    assert_eq!(
        gcode.lines().any(|line| line == "; WIPE_TOWER_START"),
        tower
    );
}

fn check_placement(gcode: &str) {
    let (before, ramming) = gcode.split_once(START).unwrap();
    let (ramming, after) = ramming.split_once(END).unwrap();
    assert!(before.contains("; CP TOOLCHANGE START\n"));
    assert!(!before.lines().any(|line| line.trim_end() == "T1"));
    assert!(
        ramming.lines().any(|line| {
            line.starts_with("G1 ")
                && line
                    .split_ascii_whitespace()
                    .any(|word| word.starts_with('E'))
        }),
        "markers must bracket ramming extrusion"
    );
    assert!(after.lines().any(|line| line.trim_end() == "T1"));
}

fn compare(project: &[u8], label: &str, (cross_head, tower): (bool, bool)) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let native = NativeCase::slice(&runner, project, label);
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(native.root.join("ares.api.gcode"), &api).unwrap();
    let api = std::str::from_utf8(&api).unwrap();
    let map = if cross_head {
        "; filament_nozzle_map = 1,0"
    } else {
        "; filament_nozzle_map = 0,0"
    };
    check_setup(&native.native, (map, tower));
    check_setup(api, (map, tower));
    let expected: &[&str] = if cross_head && tower {
        &[START, END]
    } else {
        &[]
    };
    let native_markers = markers(&native.native);
    assert_eq!(native_markers, expected, "Native: {:?}", native.root);
    assert_eq!(markers(api), native_markers, "API: {:?}", native.root);
    assert!(
        !api.lines()
            .any(|line| matches!(line, "; Nozzle change start" | "; Nozzle change end"))
    );
    if cross_head && tower {
        check_placement(&native.native);
        check_placement(api);
    }
}

#[test]
fn type1_marks_cross_head_ramming_in_an_arc_enabled_project() {
    compare(
        include_bytes!("../../../../../tests/parity/type1-nozzle-change-markers/cross-head.3mf"),
        "type1-nozzle-cross-head-",
        (true, true),
    );
}

#[test]
fn type1_does_not_mark_same_head_filament_changes() {
    compare(
        include_bytes!("../../../../../tests/parity/type1-nozzle-change-markers/same-head.3mf"),
        "type1-nozzle-same-head-",
        (false, true),
    );
}

#[test]
fn type1_does_not_mark_cross_head_changes_without_the_tower() {
    compare(
        include_bytes!("../../../../../tests/parity/type1-nozzle-change-markers/tower-off.3mf"),
        "type1-nozzle-tower-off-",
        (true, false),
    );
}
