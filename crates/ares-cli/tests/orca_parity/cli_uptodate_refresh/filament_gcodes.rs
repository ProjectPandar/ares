//! Whole-G-code file-driven retained-filament checks (`OrcaSlicer.cpp:2701–2767,3510–3811`).
use std::{fs, path::Path, process::Command};

use serde_json::json;

use super::{
    FILAMENT,
    gcodes_support::{
        COLUMNS, REFRESH, SKIP, columns, compare_toggle, config, decode_strings, edit_project,
        pair, pair_on, read_json, repack_file, same, stock_resources,
    },
    {OrcaRunner, artifact_dir, project},
};
use crate::runner::repo_root;

const SAVED_START: &str = "M117 SAVED FILAMENT START\n";
const SAVED_END: &str = "M117 SAVED FILAMENT END\n";
const SOURCE_START: &str = "M117 SOURCE FILAMENT START\n";
const SOURCE_END: &str = "M117 SOURCE FILAMENT END\n";
const REPLACED_START: &str = "M117 REPLACED FILAMENT START\n";
const REPLACED_END: &str = "M117 REPLACED FILAMENT END\n";
const START: &str = "filament_start_gcode";
const END: &str = "filament_end_gcode";
const KEPT: [&str; 3] = [
    "bottom_color_penetration_layers",
    "filament_density;filament_end_gcode;filament_start_gcode",
    "extruder_clearance_dist_to_rod",
];
const RELEASED: [&str; 3] = [
    "bottom_color_penetration_layers",
    "filament_density",
    "extruder_clearance_dist_to_rod",
];

#[test]
fn retained_filament_gcode_omitted_flag_control() {
    run(|runner, root| {
        let gcode = pair(runner, root, "omitted", REFRESH, &[]);
        retained(&gcode, SAVED_START, SAVED_END, KEPT);
    });
}

#[test]
fn skip_modified_gcodes_refreshes_retained_filament_scripts() {
    run(|runner, root| {
        let omitted = pair(runner, root, "omitted", REFRESH, &[]);
        retained(&omitted, SAVED_START, SAVED_END, KEPT);
        let off = pair(
            runner,
            root,
            "false",
            &[REFRESH, &["--skip-modified-gcodes=false"]].concat(),
            &[],
        );
        same(&omitted, &off, root);
        let enabled = pair(runner, root, "enabled", &[REFRESH, &[SKIP]].concat(), &[]);
        retained(&enabled, SOURCE_START, SOURCE_END, RELEASED);
    });
}

#[test]
fn empty_and_absent_filament_source_scripts_follow_default_rules() {
    // OrcaSlicer.cpp:710–729,3590: [""] stays "", [] resizes to "", absent defaults to " ".
    for (label, value, expected) in [
        ("empty-string", Some(json!([""])), ""),
        ("empty-array", Some(json!([])), ""),
        ("absent", None, " "),
    ] {
        run(|runner, root| {
            let mut file = read_json(root, "filament.json");
            set_source_scripts(&mut file, value.as_ref());
            fs::write(root.join("filament.json"), file.to_string()).unwrap();
            let omitted = pair(runner, root, &format!("omitted-{label}"), REFRESH, &[]);
            retained(&omitted, SAVED_START, SAVED_END, KEPT);
            let enabled = pair(
                runner,
                root,
                &format!("enabled-{label}"),
                &[REFRESH, &[SKIP]].concat(),
                &[],
            );
            retained(&enabled, expected, expected, RELEASED);
        });
    }
}

fn set_source_scripts(file: &mut serde_json::Value, value: Option<&serde_json::Value>) {
    let object = file.as_object_mut().unwrap();
    for key in [START, END] {
        if let Some(script) = value {
            object.insert(key.to_owned(), script.clone());
        } else {
            object.remove(key);
        }
    }
}

#[test]
fn explicit_filament_replacement_is_not_retained() {
    run(|runner, root| {
        let mut replacement = read_json(root, "filament.json");
        replacement[START] = json!([REPLACED_START]);
        replacement[END] = json!([REPLACED_END]);
        fs::write(root.join("replacement.json"), replacement.to_string()).unwrap();
        let args = [REFRESH, &["--load-filaments", "replacement.json"]].concat();
        let gcode = compare_toggle(runner, root, "retained.3mf", &args, &[]);
        assert_scripts(&gcode, REPLACED_START, REPLACED_END);
        assert_eq!(
            columns(&gcode),
            [
                "bottom_color_penetration_layers;enable_prime_tower",
                "",
                "extruder_clearance_dist_to_rod",
            ]
        );
        assert_eq!(config(&gcode, "filament_density"), "1.35");
        assert_eq!(config(&gcode, "extruder_clearance_dist_to_rod"), "44");
    });
}

#[test]
fn multi_filament_file_refresh_updates_only_modified_slot_column() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    let base =
        fs::read(repo_root().join("tests/parity/bbl-layer-object-labels/one-object.3mf")).unwrap();
    let project_path = root.join("two-slot.3mf");
    let kept = [
        "",
        "filament_density;filament_start_gcode",
        "filament_density;filament_end_gcode",
        "",
    ];
    let released = ["", "filament_density", "filament_density", ""];
    repack_file(&base, &project_path, |saved| {
        saved["filament_density"] = json!(["1.31", "1.32"]);
        saved[START] = json!(["M117 SAVED SLOT 1\n", "M117 UNMARKED SLOT 2\n"]);
        saved[END] = json!(["M117 UNMARKED END 1\n", "M117 SAVED END 2\n"]);
        saved[COLUMNS] = json!(kept);
    });
    for (file, name, id, start, end, density) in [
        (
            "fil1.json",
            "Bambu PLA Basic @BBL X1C",
            "GFA00",
            "M117 SOURCE SLOT 1\n",
            "M117 SOURCE END 1\n",
            "1.41",
        ),
        (
            "fil2.json",
            "Bambu PLA Matte @BBL X1C",
            "GFA01",
            "M117 SOURCE SLOT 2\n",
            "M117 SOURCE END 2\n",
            "1.42",
        ),
    ] {
        let preset = json!({
            "type": "filament",
            "name": name,
            "from": "system",
            "instantiation": "true",
            "version": "2.5.0",
            "filament_id": id,
            "filament_extruder_variant": ["Direct Drive Standard"],
            "filament_density": [density],
            "filament_start_gcode": [start],
            "filament_end_gcode": [end],
        });
        fs::write(root.join(file), preset.to_string()).unwrap();
    }
    let resources = stock_resources();
    let ares_only = ["--resources-dir", resources.to_str().unwrap()];
    let args = ["--uptodate", "--uptodate-filaments", "fil1.json;fil2.json"];
    let omitted = pair_on(
        (&runner, &root),
        "two-slot.3mf",
        "omitted",
        &args,
        &ares_only,
    );
    assert_eq!(columns(&omitted), kept);
    assert_eq!(
        decode_strings(&omitted, START),
        ["M117 SAVED SLOT 1\n", "M117 SOURCE SLOT 2\n"]
    );
    assert_eq!(
        decode_strings(&omitted, END),
        ["M117 SOURCE END 1\n", "M117 SAVED END 2\n"]
    );

    let enabled = pair_on(
        (&runner, &root),
        "two-slot.3mf",
        "enabled",
        &[args.as_slice(), &[SKIP]].concat(),
        &ares_only,
    );
    assert_eq!(columns(&enabled), released);
    assert_eq!(
        decode_strings(&enabled, START),
        ["M117 SOURCE SLOT 1\n", "M117 SOURCE SLOT 2\n"]
    );
    assert_eq!(
        decode_strings(&enabled, END),
        ["M117 SOURCE END 1\n", "M117 SOURCE END 2\n"]
    );
    assert_eq!(config(&enabled, "filament_density"), "1.31,1.32");
}

#[test]
fn selected_plate_refreshes_retained_gcodes_with_skip_flag() {
    run(|runner, root| {
        let argv = [REFRESH, &[SKIP]].concat();
        let native_dir = root.join("native-plate");
        let ares_dir = root.join("ares-plate");
        fs::create_dir(&native_dir).unwrap();
        fs::create_dir(&ares_dir).unwrap();
        let native = Command::new(runner.bin())
            .current_dir(root)
            .args(["--slice", "1", "--outputdir"])
            .arg(&native_dir)
            .args(&argv)
            .arg("retained.3mf")
            .output()
            .unwrap();
        assert!(native.status.success(), "Native failed: {root:?}");
        let actual = Command::new(env!("CARGO_BIN_EXE_ares"))
            .current_dir(root)
            .args(["slice", "--slice", "1", "--outputdir"])
            .arg(&ares_dir)
            .args(&argv)
            .arg("retained.3mf")
            .output()
            .unwrap();
        assert!(
            actual.status.success(),
            "Ares failed: {}; {root:?}",
            String::from_utf8_lossy(&actual.stderr)
        );
        let native_gcode = fs::read(native_dir.join("plate_1.gcode")).unwrap();
        let ares_gcode = fs::read(ares_dir.join("plate_1.gcode")).unwrap();
        retained(&native_gcode, SOURCE_START, SOURCE_END, RELEASED);
        same(&native_gcode, &ares_gcode, &ares_dir);
    });
}

fn run(test: impl FnOnce(&OrcaRunner, &Path)) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    project(&root);
    edit_project(&root, |saved| {
        assert_eq!(saved["filament_settings_id"], json!([FILAMENT]));
        saved[START] = json!([SAVED_START]);
        saved[END] = json!([SAVED_END]);
        saved[COLUMNS] = json!(KEPT);
    });
    let mut file = read_json(&root, "filament.json");
    file[START] = json!([SOURCE_START]);
    file[END] = json!([SOURCE_END]);
    fs::write(root.join("filament.json"), file.to_string()).unwrap();
    test(&runner, &root);
}

fn retained(gcode: &[u8], start: &str, end: &str, dirty: [&str; 3]) {
    assert_scripts(gcode, start, end);
    assert_eq!(columns(gcode), dirty);
    assert_eq!(config(gcode, "filament_density"), "1.25");
    assert_eq!(config(gcode, "bottom_color_penetration_layers"), "6");
    assert_eq!(config(gcode, "extruder_clearance_dist_to_rod"), "44");
}

fn assert_scripts(gcode: &[u8], start: &str, end: &str) {
    let text = std::str::from_utf8(gcode).unwrap();
    let pre_config = text.split_once("; CONFIG_BLOCK_START").unwrap().0;
    assert_eq!(decode_strings(gcode, START), [start]);
    assert_eq!(decode_strings(gcode, END), [end]);
    for script in [start, end] {
        for line in script.lines().filter(|line| !line.trim().is_empty()) {
            assert!(
                pre_config.lines().any(|candidate| candidate == line),
                "Native did not emit {line:?}"
            );
        }
    }
    if start != SAVED_START {
        assert!(!pre_config.contains(SAVED_START));
    }
    if end != SAVED_END {
        assert!(!pre_config.contains(SAVED_END));
    }
}
