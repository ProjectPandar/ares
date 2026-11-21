use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};

use crate::{
    OrcaRunner,
    runner::{application, native_api_cli::NativeCase},
};

const START: &str = "; NOZZLE_CHANGE_START OF0 NF1 ON1 NN0";
const END: &str = "; NOZZLE_CHANGE_END OF0 NF1 ON1 NN0";
const CROSS_HEAD: &[u8] =
    include_bytes!("../../../../../tests/parity/type1-nozzle-change-markers/cross-head.3mf");
const SAME_HEAD: &[u8] =
    include_bytes!("../../../../../tests/parity/type1-nozzle-change-markers/same-head.3mf");
const TOWER_OFF: &[u8] =
    include_bytes!("../../../../../tests/parity/type1-nozzle-change-markers/tower-off.3mf");

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

fn slice_pair(project: &[u8], label: &str) -> Option<(NativeCase, String)> {
    let runner = OrcaRunner::from_env().unwrap()?;
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
    Some((native, String::from_utf8(api).unwrap()))
}

fn compare(project: &[u8], label: &str, (cross_head, tower): (bool, bool)) {
    let Some((native, api)) = slice_pair(project, label) else {
        return;
    };
    let api = api.as_str();
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
    compare(CROSS_HEAD, "type1-nozzle-cross-head-", (true, true));
}

#[test]
fn type1_does_not_mark_same_head_filament_changes() {
    compare(SAME_HEAD, "type1-nozzle-same-head-", (false, true));
}

#[test]
fn type1_does_not_mark_cross_head_changes_without_the_tower() {
    compare(TOWER_OFF, "type1-nozzle-tower-off-", (true, false));
}

const HEAT: [&str; 4] = [
    "M632 S1 W",
    "M400",
    "M104 T1 S220 N0 ;Multi extruder pre heating",
    "M633",
];
const COOL: [&str; 2] = ["M400", "M104 T0 S25 N0 ;Multi extruder pre cooling"];

fn command_index(lines: &[&str], pattern: &[&str]) -> usize {
    let mut hits = lines
        .windows(pattern.len())
        .enumerate()
        .filter_map(|(index, window)| (window == pattern).then_some(index));
    let index = hits.next().expect("missing thermal command sequence");
    assert!(hits.next().is_none(), "duplicate thermal command sequence");
    index
}

#[test]
fn type1_cross_head_ramming_bounds_idle_hotend_temperatures() {
    let Some((native, api)) = slice_pair(CROSS_HEAD, "type1-nozzle-thermal-cross-head-") else {
        return;
    };
    for (engine, gcode) in [("Native", native.native.as_str()), ("API", api.as_str())] {
        let lines: Vec<_> = gcode.lines().collect();
        let heat = command_index(&lines, &HEAT);
        let cool = command_index(&lines, &COOL);
        let start = lines.iter().position(|line| *line == START).unwrap();
        let end = lines.iter().position(|line| *line == END).unwrap();
        let select = lines.iter().position(|line| *line == "T1").unwrap();
        assert!(heat < start && end < select, "{engine}: {:?}", native.root);
        assert_eq!(
            lines[cool - 1],
            END,
            "{engine}: cool the outgoing hotend at the end of ramming, before logical tool selection: {:?}",
            native.root
        );
    }
}

#[test]
fn type1_same_head_changes_do_not_inject_idle_hotend_temperatures() {
    let Some((native, api)) = slice_pair(SAME_HEAD, "type1-nozzle-thermal-same-head-") else {
        return;
    };
    for gcode in [native.native, api] {
        assert!(gcode.lines().any(|line| line == "T1"));
        assert!(!gcode.contains(";Multi extruder pre "), "{:?}", native.root);
    }
}

#[test]
fn type1_tower_off_retains_filament_based_hotend_temperature_placement() {
    let Some((native, api)) = slice_pair(TOWER_OFF, "type1-nozzle-thermal-tower-off-") else {
        return;
    };
    let anchors = |gcode: &str| {
        let lines: Vec<_> = gcode.lines().collect();
        let heat = command_index(&lines, &HEAT);
        let cool = command_index(&lines, &COOL);
        assert_eq!(lines[cool - 1], "T1");
        assert!(!gcode.contains("; NOZZLE_CHANGE_"));
        [
            lines[heat - 1].to_owned(),
            lines[heat + HEAT.len()].to_owned(),
        ]
    };
    assert_eq!(anchors(&api), anchors(&native.native), "{:?}", native.root);
}
