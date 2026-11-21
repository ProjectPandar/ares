//! Native retained-preset file routing (`OrcaSlicer.cpp:2539–2768`).
#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
    process::{Command, Output},
};

use serde_json::{Value, json};

use crate::{
    load_settings_3mf::load_settings_3mf_fixture,
    load_settings_cli::{banner_normalized, stages_run},
    runner::{OrcaRunner, application::settings},
};

const MACHINE: &str = "Prusa MK3S 0.4 nozzle";
const PROCESS: &str = "0.20mm Quality @MK3S";
const FILAMENT: &str = "Prusa Generic PLA";
const KEYS: [&str; 3] = [
    "extruder_clearance_dist_to_rod",
    "bottom_color_penetration_layers",
    "filament_density",
];

#[test]
fn refresh_files_update_retained_values_without_replacing_preset_names() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = &artifact_dir();
    project(root);
    let lists = [
        "--uptodate-settings",
        "machine.json;process.json",
        "--uptodate-filaments",
        "filament.json",
    ];
    let baseline = native_gcode((&runner, root), "baseline", &[]);
    let native = native_gcode(
        (&runner, root),
        "native",
        &[["--uptodate"].as_slice(), &lists].concat(),
    );
    values(&baseline, ["44", "6", "1.25"]);
    values(&native, ["45", "8", "1.35"]);
    let output = ares(
        (root, "ares"),
        &[["--uptodate"].as_slice(), &lists].concat(),
    );
    assert!(
        output.status.success(),
        "refresh argv: {}; {root:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = fs::read(root.join("ares/plate_1.gcode")).unwrap();
    let matches = banner_normalized(&native) == banner_normalized(&actual);
    assert!(matches, "whole G-code differs; {root:?}");

    // Omitting the lists picks the same flat versions from the resource root.
    for (kind, name) in [
        ("machine", MACHINE),
        ("process", PROCESS),
        ("filament", FILAMENT),
    ] {
        let dir = root.join(format!("resources/profiles/BBL/{kind}_full"));
        fs::create_dir_all(&dir).unwrap();
        fs::copy(
            root.join(format!("{kind}.json")),
            dir.join(format!("{name}.json")),
        )
        .unwrap();
    }
    let output = ares(
        (root, "resource-ares"),
        &["--uptodate", "--resources-dir", "resources"],
    );
    assert!(
        output.status.success(),
        "resource lookup: {}; {root:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = fs::read(root.join("resource-ares/plate_1.gcode")).unwrap();
    let matches = banner_normalized(&native) == banner_normalized(&actual);
    assert!(matches, "resource lookup whole G-code differs; {root:?}");

    let ignored = ares(
        (root, "without-uptodate"),
        &[
            "--uptodate-settings",
            "missing-settings.json",
            "--uptodate-filaments",
            "missing-filaments.json",
        ],
    );
    assert!(
        ignored.status.success(),
        "inactive lists: {}; {root:?}",
        String::from_utf8_lossy(&ignored.stderr)
    );
    let unrefreshed = fs::read(root.join("without-uptodate/plate_1.gcode")).unwrap();
    assert!(
        banner_normalized(&baseline) == banner_normalized(&unrefreshed),
        "inactive lists changed G-code; {root:?}"
    );
}

#[test]
fn uptodate_without_catalog_slices_saved_project() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = &artifact_dir();
    project(root);
    native_gcode((&runner, root), "native", &["--uptodate"]);
    let output = ares((root, "ares"), &["--uptodate"]);
    assert!(
        output.status.success(),
        "bare --uptodate: {}; {root:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("ares/plate_1.gcode").is_file(), "{root:?}");
}

#[test]
fn refresh_filament_list_checks_count_before_opening_files() {
    rejects(
        "--uptodate-filaments",
        "missing.json;also-missing.json",
        -2,
        "uptodate_filaments size 2 not equal to filament_count 1",
    );
}

#[test]
fn refresh_machine_checks_retained_system_name() {
    rejects(
        "--uptodate-settings",
        "wrong-machine.json",
        -5,
        "wrong machine config file Other machine loaded, current machine config name Prusa MK3S 0.4 nozzle",
    );
}

#[test]
fn refresh_process_checks_retained_system_name() {
    rejects(
        "--uptodate-settings",
        "wrong-process.json",
        -5,
        "wrong process config file Other process loaded, current process config name 0.20mm Quality @MK3S",
    );
}

#[test]
fn refresh_settings_rejects_filament_type() {
    rejects(
        "--uptodate-settings",
        "filament.json",
        -5,
        "found invalid config type filament from config filament.json",
    );
}

#[test]
fn refresh_filament_checks_slot_system_name() {
    rejects(
        "--uptodate-filaments",
        "wrong-filament.json",
        -5,
        "wrong filament config file Other filament loaded, current filament config name Prusa Generic PLA, index 0",
    );
}

#[test]
fn refresh_filament_rejects_process_type() {
    rejects(
        "--uptodate-filaments",
        "process.json",
        -5,
        "unknown config type process of file process.json in load-filaments",
    );
}

pub(crate) fn artifact_dir() -> std::path::PathBuf {
    tempfile::Builder::new()
        .prefix("cli-refresh-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep()
}

pub(crate) fn project(root: &Path) {
    let input = fs::read(load_settings_3mf_fixture("compatible-input.3mf")).unwrap();
    let mut saved = settings(&input).unwrap();
    for (key, value) in KEYS
        .into_iter()
        .zip([json!("44"), json!("6"), json!(["1.25"])])
    {
        saved.insert(key.to_owned(), value);
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(fs::File::create(root.join("retained.3mf")).unwrap());
    for index in 0..zip.len() {
        let file = zip.by_index(index).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            output
                .start_file(file.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(&serde_json::to_vec(&saved).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(file).unwrap();
        }
    }
    output.finish().unwrap();
    for (kind, name, key, value) in [
        ("machine", MACHINE, KEYS[0], json!("45")),
        ("process", PROCESS, KEYS[1], json!("8")),
        ("filament", FILAMENT, KEYS[2], json!(["1.35"])),
    ] {
        let mut preset = json!({"type": kind, "name": name, "from": "system", "instantiation": "true", "version": "2.5.0", key: value});
        let copy = match kind {
            "machine" => &[
                "printable_area",
                "printable_height",
                "printer_model",
                "printer_variant",
                "printer_extruder_id",
                "printer_extruder_variant",
                "machine_start_gcode",
                "machine_end_gcode",
                "machine_pause_gcode",
                "before_layer_change_gcode",
                "layer_change_gcode",
                "default_filament_profile",
            ][..],
            "process" => &["print_extruder_id", "print_extruder_variant"][..],
            _ => &[
                "filament_extruder_variant",
                "filament_start_gcode",
                "filament_end_gcode",
            ][..],
        };
        for key in copy {
            if let Some(value) = saved.get(*key) {
                preset[*key] = value.clone();
            }
        }
        if kind == "process" {
            preset["ironing_expansion"] = json!("0");
            preset["plugins"] = json!("");
            preset["compatible_printers"] = saved["print_compatible_printers"].clone();
        } else if kind == "filament" {
            preset["filament_id"] = json!("GFL99");
        } else {
            // As in a flattened machine resource, include collection metadata
            // absent from older embedded profiles.
            preset
                .as_object_mut()
                .unwrap()
                .extend(serde_json::Map::from_iter([
                    ("bbl_use_printhost".into(), json!("0")),
                    ("default_bed_type".into(), json!("")),
                    ("default_nozzle_volume_type".into(), json!(["Standard"])),
                    (
                        "extruder_variant_list".into(),
                        json!(["Direct Drive Standard"]),
                    ),
                    ("flashforge_serial_number".into(), json!("")),
                    ("group_algo_with_time".into(), json!("0")),
                    ("pellet_modded_printer".into(), json!("0")),
                    ("printer_agent".into(), json!("")),
                    ("printer_plugin_config_overrides".into(), json!("")),
                    ("printhost_authorization_type".into(), json!("key")),
                    ("printhost_ssl_ignore_revoke".into(), json!("0")),
                    ("thumbnails_format".into(), json!("PNG")),
                ]));
        }
        fs::write(root.join(format!("{kind}.json")), preset.to_string()).unwrap();
        preset["name"] = json!(format!("Other {kind}"));
        fs::write(root.join(format!("wrong-{kind}.json")), preset.to_string()).unwrap();
    }
}

pub(crate) fn native_gcode(
    (runner, root): (&OrcaRunner, &Path),
    dir: &str,
    argv: &[&str],
) -> Vec<u8> {
    let args = [
        ["--slice", "0", "--arrange", "0", "--outputdir", dir].as_slice(),
        argv,
        &["retained.3mf"],
    ]
    .concat();
    stages_run(runner, root, &args);
    fs::read(root.join(dir).join("plate_1.gcode")).unwrap()
}

fn values(gcode: &[u8], expected: [&str; 3]) {
    let text = std::str::from_utf8(gcode).unwrap();
    for (key, value) in KEYS.into_iter().zip(expected).chain([
        ("printer_settings_id", MACHINE),
        ("print_settings_id", PROCESS),
        ("filament_settings_id", "\"Prusa Generic PLA\""),
    ]) {
        let line = format!("; {key} = {value}");
        assert!(
            text.lines().any(|candidate| candidate == line),
            "missing Native value: {line}"
        );
    }
}

pub(crate) fn ares((root, dir): (&Path, &str), argv: &[&str]) -> Output {
    fs::create_dir(root.join(dir)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(root)
        .arg("slice")
        .args(argv)
        .arg("-o")
        .arg(root.join(dir).join("plate_1.gcode"))
        .arg("retained.3mf")
        .output()
        .unwrap();
    fs::write(root.join(dir).join("stderr"), &output.stderr).unwrap();
    output
}

fn rejects(flag: &str, list: &str, code: i32, diagnostic: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = &artifact_dir();
    project(root);
    fs::create_dir(root.join("native")).unwrap();
    let argv = ["--uptodate", flag, list];
    let native = Command::new(runner.bin())
        .current_dir(root)
        .args(argv)
        .args(["--slice", "0", "--outputdir", "native", "retained.3mf"])
        .output()
        .unwrap();
    for (dir, output) in [("native", native), ("ares", ares((root, "ares"), &argv))] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        fs::write(root.join(dir).join("stderr"), &output.stderr).unwrap();
        fs::write(root.join(dir).join("stdout"), &output.stdout).unwrap();
        assert_eq!(
            output.status.code(),
            Some(code.rem_euclid(256)),
            "{dir}: {stderr}{stdout}; {root:?}"
        );
        if dir == "ares" {
            assert!(
                stderr.contains(diagnostic),
                "{dir} missing {diagnostic}: {stderr}; {root:?}"
            );
        }
        let report: Value =
            serde_json::from_slice(&fs::read(root.join(dir).join("result.json")).unwrap()).unwrap();
        assert_eq!(report["return_code"], code);
        assert!(!root.join(dir).join("plate_1.gcode").exists());
    }
    let report = |dir: &str| -> Value {
        serde_json::from_slice(&fs::read(root.join(dir).join("result.json")).unwrap()).unwrap()
    };
    assert_eq!(
        report("ares")["error_string"],
        report("native")["error_string"]
    );
}
