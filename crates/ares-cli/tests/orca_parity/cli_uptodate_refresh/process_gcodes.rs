//! Whole-G-code retained-process checks for `OrcaSlicer.cpp:3345–3406`.
use std::{fs, path::Path};

use serde_json::json;

use super::{
    PROCESS,
    gcodes_support::{
        COLUMNS, REFRESH, SKIP, columns, compare_toggle, config, decode_string, edit_project, pair,
        read_json, same, stock_resources,
    },
    {OrcaRunner, artifact_dir, project},
};

const SAVED: &str = "M117 SAVED PROCESS\n";
const SOURCE: &str = "M117 SOURCE PROCESS\n";
const REPLACEMENT: &str = "M117 REPLACEMENT PROCESS\n";
const MACHINE_SAVED: &str = "M117 SAVED MACHINE\n";
const MACHINE_SOURCE: &str = "M117 SOURCE MACHINE\n";
const SCRIPT: &str = "layer_change_gcode";
const MACHINE_SCRIPT: &str = "machine_start_gcode";
const KEPT: [&str; 3] = [
    "bottom_color_penetration_layers;layer_change_gcode",
    "",
    "extruder_clearance_dist_to_rod",
];
const RELEASED: [&str; 3] = [
    "bottom_color_penetration_layers",
    "",
    "extruder_clearance_dist_to_rod",
];

#[test]
fn retained_process_gcode_omitted_flag_control() {
    run(|runner, root| {
        let gcode = pair(runner, root, "omitted", REFRESH, &[]);
        retained(&gcode, SAVED, KEPT);
    });
}

#[test]
fn skip_modified_gcodes_refreshes_retained_process_script() {
    run(|runner, root| {
        let gcode = pair(runner, root, "enabled", &[REFRESH, &[SKIP]].concat(), &[]);
        retained(&gcode, SOURCE, RELEASED);
    });
}

#[test]
fn skip_modified_gcodes_updates_both_machine_and_process_columns() {
    run(|runner, root| {
        let mut dirty = KEPT;
        dirty[2] = "extruder_clearance_dist_to_rod;machine_start_gcode";
        edit_project(root, |saved| {
            saved[MACHINE_SCRIPT] = json!(MACHINE_SAVED);
            saved[COLUMNS] = json!(dirty);
        });
        let mut machine = read_json(root, "machine.json");
        machine[MACHINE_SCRIPT] = json!(MACHINE_SOURCE);
        fs::write(root.join("machine.json"), machine.to_string()).unwrap();

        let omitted = pair(runner, root, "omitted", REFRESH, &[]);
        retained(&omitted, SAVED, dirty);
        assert_eq!(decode_string(&omitted, MACHINE_SCRIPT), MACHINE_SAVED);

        let enabled = pair(runner, root, "enabled", &[REFRESH, &[SKIP]].concat(), &[]);
        retained(&enabled, SOURCE, RELEASED);
        assert_eq!(decode_string(&enabled, MACHINE_SCRIPT), MACHINE_SOURCE);
    });
}

#[test]
fn process_explicit_false_and_unedited_match_omission() {
    run(|runner, root| {
        let omitted = pair(runner, root, "omitted", REFRESH, &[]);
        retained(&omitted, SAVED, KEPT);
        let off = pair(
            runner,
            root,
            "false",
            &[REFRESH, &["--skip-modified-gcodes=false"]].concat(),
            &[],
        );
        same(&omitted, &off, root);
    });
    run(|runner, root| {
        edit_project(root, |saved| {
            saved[SCRIPT] = json!(SOURCE);
            saved[COLUMNS] = json!(RELEASED);
        });
        let gcode = compare_toggle(runner, root, "retained.3mf", REFRESH, &[]);
        retained(&gcode, SOURCE, RELEASED);
    });
}

#[test]
fn absent_process_source_script_keeps_the_process_column() {
    run(|runner, root| {
        let mut proc_file = read_json(root, "process.json");
        proc_file.as_object_mut().unwrap().remove(SCRIPT);
        fs::write(root.join("process.json"), proc_file.to_string()).unwrap();
        let gcode = compare_toggle(runner, root, "retained.3mf", REFRESH, &[]);
        retained(&gcode, SAVED, KEPT);
    });
}

#[test]
fn explicit_process_replacement_is_not_retained() {
    run(|runner, root| {
        let mut replacement = read_json(root, "process.json");
        replacement[SCRIPT] = json!(REPLACEMENT);
        replacement["bottom_color_penetration_layers"] = json!("9");
        fs::write(root.join("replacement.json"), replacement.to_string()).unwrap();
        let args = [REFRESH, &["--load-settings", "replacement.json"]].concat();
        let gcode = compare_toggle(runner, root, "retained.3mf", &args, &[]);
        assert_script(&gcode, REPLACEMENT);
        assert_eq!(columns(&gcode), ["", "", "extruder_clearance_dist_to_rod"]);
        assert_eq!(config(&gcode, "bottom_color_penetration_layers"), "9");
        assert_eq!(config(&gcode, "extruder_clearance_dist_to_rod"), "44");
    });
}

#[test]
fn machine_file_does_not_unprotect_catalog_process() {
    run(|runner, root| {
        let mut dirty = KEPT;
        dirty[0] = "bottom_color_penetration_layers;layer_change_gcode;post_process";
        edit_project(root, |saved| {
            saved.insert("post_process".into(), json!([]));
            saved.insert("top_color_penetration_layers".into(), json!("9"));
            saved.insert(COLUMNS.into(), json!(dirty));
        });
        let resources = stock_resources();
        let gcode = compare_toggle(
            runner,
            root,
            "retained.3mf",
            &[
                "--uptodate",
                "--uptodate-settings",
                "machine.json",
                "--uptodate-filaments",
                "filament.json",
            ],
            &["--resources-dir", resources.to_str().unwrap()],
        );
        retained(&gcode, SAVED, dirty);
        assert_ne!(config(&gcode, "top_color_penetration_layers"), "9");
    });
}

fn run(test: impl FnOnce(&OrcaRunner, &Path)) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    project(&root);
    edit_project(&root, |saved| {
        assert_eq!(saved["print_settings_id"], PROCESS);
        saved[SCRIPT] = json!(SAVED);
        saved[COLUMNS] = json!(KEPT);
    });
    let mut machine = read_json(&root, "machine.json");
    machine[SCRIPT] = json!(SAVED);
    fs::write(root.join("machine.json"), machine.to_string()).unwrap();
    let mut proc_file = read_json(&root, "process.json");
    proc_file[SCRIPT] = json!(SOURCE);
    fs::write(root.join("process.json"), proc_file.to_string()).unwrap();
    test(&runner, &root);
}

fn retained(gcode: &[u8], script: &str, dirty: [&str; 3]) {
    assert_script(gcode, script);
    assert_eq!(columns(gcode), dirty);
    assert_eq!(config(gcode, "print_settings_id"), PROCESS);
    assert_eq!(config(gcode, "bottom_color_penetration_layers"), "6");
    assert_eq!(config(gcode, "extruder_clearance_dist_to_rod"), "44");
}

fn assert_script(gcode: &[u8], script: &str) {
    let text = std::str::from_utf8(gcode).unwrap();
    let pre_config = text.split_once("; CONFIG_BLOCK_START").unwrap().0;
    assert_eq!(decode_string(gcode, SCRIPT), script);
    for line in script.lines() {
        assert!(
            pre_config.lines().any(|candidate| candidate == line),
            "Native did not emit {line:?}"
        );
    }
    if !script.contains(SAVED) {
        assert!(!pre_config.contains(SAVED));
    }
}
