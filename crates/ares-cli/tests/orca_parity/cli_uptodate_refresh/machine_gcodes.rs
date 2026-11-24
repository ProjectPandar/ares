//! Whole-G-code retained-machine checks for `OrcaSlicer.cpp:3212–3256`.
use std::{fs, path::Path};

use ares_core::decode_config_value_by_key;
use serde_json::{Value, json};

use super::{
    MACHINE, OrcaRunner, ares, artifact_dir, banner_normalized, native_gcode, project, settings,
    write_project,
};

const SAVED: &str = "M117 SAVED MACHINE\n";
const SOURCE: &str = "M117 SOURCE MACHINE\n";
const DEFAULT: &str = "G28 ; home all axes\nG1 Z5 F5000 ; lift nozzle\n";
const REPLACEMENT: &str = "M117 REPLACEMENT MACHINE\n";
const SCRIPT: &str = "machine_start_gcode";
const COLUMNS: &str = "different_settings_to_system";
const KEPT: [&str; 3] = [
    "bottom_color_penetration_layers",
    "",
    "extruder_clearance_dist_to_rod;machine_start_gcode",
];
const RELEASED: [&str; 3] = [
    "bottom_color_penetration_layers",
    "",
    "extruder_clearance_dist_to_rod",
];
const REFRESH: &[&str] = &[
    "--uptodate",
    "--uptodate-settings",
    "machine.json;process.json",
    "--uptodate-filaments",
    "filament.json",
];
const SKIP: &str = "--skip-modified-gcodes";

#[test]
fn retained_machine_gcode_omitted_flag_control() {
    run(|runner, root| {
        let gcode = pair(runner, root, "omitted", REFRESH, &[]);
        retained(&gcode, SAVED, KEPT);
    });
}

#[test]
fn skip_modified_gcodes_refreshes_retained_machine_script() {
    run(|runner, root| {
        let gcode = pair(runner, root, "enabled", &[REFRESH, &[SKIP]].concat(), &[]);
        retained(&gcode, SOURCE, RELEASED);
    });
}

#[test]
fn explicit_false_equals_omission() {
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
}

#[test]
fn unedited_script_does_not_rewrite_the_machine_column() {
    run(|runner, root| {
        edit_project(root, |saved| {
            saved[SCRIPT] = json!(SOURCE);
            saved[COLUMNS] = json!(RELEASED);
        });
        let gcode = compare_toggle(runner, root, REFRESH, &[]);
        retained(&gcode, SOURCE, RELEASED);
    });
}

#[test]
fn flag_without_refresh_preserves_the_saved_script() {
    run(|runner, root| {
        // Use the same stock fallback catalog as Native without inheriting
        // its nonempty post_process vector (an older CLI admission mismatch).
        let mut dirty = KEPT;
        dirty[0] = "bottom_color_penetration_layers;post_process";
        edit_project(root, |saved| {
            saved["post_process"] = json!([]);
            saved[COLUMNS] = json!(dirty);
        });
        let resources = stock_resources();
        let gcode = compare_toggle(
            runner,
            root,
            &[],
            &["--resources-dir", resources.to_str().unwrap()],
        );
        retained(&gcode, SAVED, dirty);
    });
}

#[test]
fn explicit_machine_replacement_is_not_retained() {
    run(|runner, root| {
        let mut replacement = machine(root);
        replacement[SCRIPT] = json!(REPLACEMENT);
        fs::write(root.join("replacement.json"), replacement.to_string()).unwrap();
        let args = [REFRESH, &["--load-settings", "replacement.json"]].concat();
        let gcode = compare_toggle(runner, root, &args, &[]);
        assert_script(&gcode, REPLACEMENT);
        let mut expected = KEPT;
        expected[2] = "";
        assert_eq!(columns(&gcode), expected);
    });
}

#[test]
fn process_file_does_not_unprotect_the_catalog_machine() {
    run(|runner, root| {
        // This value is not in the keep set. An actual fallback must replace it.
        edit_project(root, |saved| {
            saved["extruder_clearance_height_to_rod"] = json!("30")
        });
        let resources = stock_resources();
        let gcode = compare_toggle(
            runner,
            root,
            &[
                "--uptodate",
                "--uptodate-settings",
                "process.json",
                "--uptodate-filaments",
                "filament.json",
            ],
            &["--resources-dir", resources.to_str().unwrap()],
        );
        retained(&gcode, SAVED, KEPT);
        assert_ne!(config(&gcode, "extruder_clearance_height_to_rod"), "30");
    });
}

#[test]
fn empty_and_absent_source_scripts_are_distinct() {
    for source in [Some(""), None] {
        run(|runner, root| {
            let mut file = machine(root);
            if let Some(script) = source {
                file[SCRIPT] = json!(script);
            } else {
                file.as_object_mut().unwrap().remove(SCRIPT);
            }
            fs::write(root.join("machine.json"), file.to_string()).unwrap();
            let omitted = pair(runner, root, "omitted", REFRESH, &[]);
            retained(&omitted, SAVED, KEPT);
            let on = pair(runner, root, "enabled", &[REFRESH, &[SKIP]].concat(), &[]);
            retained(&on, source.unwrap_or(DEFAULT), RELEASED);
        });
    }
}

fn run(test: impl FnOnce(&OrcaRunner, &Path)) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    project(&root);
    edit_project(&root, |saved| {
        assert_eq!(saved["printer_settings_id"], MACHINE);
        saved[SCRIPT] = json!(SAVED);
        saved[COLUMNS] = json!(KEPT);
    });
    let mut source = machine(&root);
    source[SCRIPT] = json!(SOURCE);
    fs::write(root.join("machine.json"), source.to_string()).unwrap();
    test(&runner, &root);
}

fn edit_project(root: &Path, edit: impl FnOnce(&mut serde_json::Map<String, Value>)) {
    let input = fs::read(root.join("retained.3mf")).unwrap();
    let mut saved = settings(&input).unwrap();
    edit(&mut saved);
    write_project(root, input, &saved);
}

fn stock_resources() -> std::path::PathBuf {
    crate::runner::repo_root().join("OrcaSlicer/resources")
}

fn machine(root: &Path) -> Value {
    serde_json::from_slice(&fs::read(root.join("machine.json")).unwrap()).unwrap()
}

fn compare_toggle(runner: &OrcaRunner, root: &Path, argv: &[&str], ares_only: &[&str]) -> Vec<u8> {
    let off = pair(runner, root, "omitted", argv, ares_only);
    let on = pair(
        runner,
        root,
        "enabled",
        &[argv, &[SKIP]].concat(),
        ares_only,
    );
    same(&off, &on, root);
    on
}

fn pair(runner: &OrcaRunner, root: &Path, row: &str, argv: &[&str], ares_only: &[&str]) -> Vec<u8> {
    let native = native_gcode((runner, root), &format!("native-{row}"), argv);
    let dir = format!("ares-{row}");
    let argv = [argv, ares_only].concat();
    let output = ares((root, &dir), &argv);
    let out = root.join(&dir);
    fs::write(out.join("stdout"), &output.stdout).unwrap();
    fs::write(out.join("status"), output.status.to_string()).unwrap();
    let gcode = out.join("plate_1.gcode");
    let exact = [
        &[env!("CARGO_BIN_EXE_ares"), "slice"][..],
        &argv,
        &["-o", gcode.to_str().unwrap(), "retained.3mf"],
    ]
    .concat();
    fs::write(out.join("argv.json"), serde_json::to_vec(&exact).unwrap()).unwrap();
    assert!(
        output.status.success(),
        "Ares {row}: {}; {root:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    same(&native, &fs::read(gcode).unwrap(), &out);
    native
}

fn same(left: &[u8], right: &[u8], evidence: &Path) {
    assert!(
        banner_normalized(left) == banner_normalized(right),
        "whole G-code differs; {evidence:?}"
    );
}

fn retained(gcode: &[u8], script: &str, dirty: [&str; 3]) {
    assert_script(gcode, script);
    assert_eq!(columns(gcode), dirty);
    assert_eq!(config(gcode, "printer_settings_id"), MACHINE);
    assert_eq!(config(gcode, "extruder_clearance_dist_to_rod"), "44");
    assert_eq!(config(gcode, "bottom_color_penetration_layers"), "6");
}

fn assert_script(gcode: &[u8], script: &str) {
    let text = std::str::from_utf8(gcode).unwrap();
    let pre_config = text.split_once("; CONFIG_BLOCK_START").unwrap().0;
    let actual: String =
        serde_json::from_str(&decode_config_value_by_key(SCRIPT, config(gcode, SCRIPT)).unwrap())
            .unwrap();
    assert_eq!(actual, script);
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

fn columns(gcode: &[u8]) -> Vec<String> {
    serde_json::from_str(&decode_config_value_by_key(COLUMNS, config(gcode, COLUMNS)).unwrap())
        .unwrap()
}

fn config<'a>(gcode: &'a [u8], key: &str) -> &'a str {
    let prefix = format!("; {key} = ");
    std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
}
