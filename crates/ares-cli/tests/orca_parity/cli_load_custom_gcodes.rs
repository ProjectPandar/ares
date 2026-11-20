#![cfg(target_os = "linux")]

use std::{
    ffi::OsString,
    fs,
    io::{self, Cursor, Write},
    path::Path,
};

use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use super::{
    cli_no_check::{gcode, inventory, run},
    direct_stl_slice::block_stl,
};
use crate::{
    artifacts, load_settings_cli::preset_files, native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

const IMPORTED: &str = "M117 JSON layer event";
const ARCHIVED: &str = "M117 archived layer event";

#[test]
fn cli_load_custom_gcodes_inserts_event_into_stl_slice() {
    compare(false, Some(0));
}

#[test]
fn cli_load_custom_gcodes_replaces_events_in_saved_project() {
    compare(true, Some(0));
}

#[test]
fn cli_load_custom_gcodes_rejects_empty_json() {
    compare(false, Some(251));
}

#[test]
fn cli_load_custom_gcodes_reports_missing_file() {
    compare(false, None);
}

#[test]
fn cli_load_custom_gcodes_multiasingle_toolchange_switches_filament() {
    compare_multiasingle_toolchange("MultiAsSingle", true);
}

#[test]
fn cli_load_custom_gcodes_multiextruder_toolchange_keeps_object_filament() {
    compare_multiasingle_toolchange("MultiExtruder", false);
}

fn compare(saved: bool, status: Option<i32>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-load-custom-gcodes-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let settings = presets[..2].join(";");
    let filaments = presets[2..].join(";");
    let stl = root.join("block-0.stl");
    fs::write(&stl, block_stl(108.0)).unwrap();
    let stl_args = ["--load-settings", &settings, "--load-filaments", &filaments];
    let project = root.join("saved.3mf");
    if saved {
        let export = [
            "--debug",
            "0",
            "--outputdir",
            "out",
            "--export-3mf",
            "saved.3mf",
        ]
        .into_iter()
        .chain(stl_args)
        .chain([stl.to_str().unwrap()])
        .map(OsString::from)
        .collect::<Vec<_>>();
        assert!(
            run(runner.bin(), &root, "export", &export).status.success(),
            "{root:?}"
        );
        fs::copy(root.join("export/cwd/out/saved.3mf"), &project).unwrap();
        seed_archived_event(&project);
        let baseline = [
            "--debug",
            "0",
            "--outputdir",
            "out",
            "--slice",
            "1",
            project.to_str().unwrap(),
        ]
        .map(OsString::from);
        assert!(
            run(runner.bin(), &root, "baseline", &baseline)
                .status
                .success(),
            "{root:?}"
        );
        assert!(
            String::from_utf8(gcode(&root.join("baseline/cwd")))
                .unwrap()
                .contains(ARCHIVED),
            "{root:?}"
        );
    }
    let events = root.join("events.json");
    if status == Some(0) {
        fs::write(&events, serde_json::to_vec(&json!({
            "mode": "SingleExtruder",
            "gcodes": [{"type": "Custom", "print_z": 0.4, "color": "", "extruder": 1, "extra": IMPORTED}],
        })).unwrap()).unwrap();
    } else if status.is_some() {
        fs::write(&events, b"").unwrap();
    }
    let mut argv = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        "1",
        "--load-custom-gcodes",
        events.to_str().unwrap(),
    ]
    .map(OsString::from)
    .to_vec();
    if saved {
        argv.push(project.into());
    } else {
        argv.extend(stl_args.map(OsString::from));
        argv.push(stl.into());
    }
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    let native_cwd = root.join("native/cwd");
    let native_report = fs::read(native_cwd.join("out/result.json")).unwrap();
    let report: Value = serde_json::from_slice(&native_report).unwrap();
    let expected = status.unwrap_or(253);
    assert_eq!(native.status.code(), Some(expected), "{root:?}");
    assert_eq!(
        report["return_code"],
        if expected == 0 { 0 } else { expected - 256 }
    );
    let native_gcode = if expected == 0 {
        assert_eq!(
            inventory(&native_cwd),
            ["out/", "out/plate_1.gcode", "out/result.json"]
        );
        let code = gcode(&native_cwd);
        let text = std::str::from_utf8(&code).unwrap();
        assert_eq!(text.matches(IMPORTED).count(), 1, "{root:?}");
        assert!(!text.contains(ARCHIVED), "{root:?}");
        Some(code)
    } else {
        assert_eq!(inventory(&native_cwd), ["out/", "out/result.json"]);
        assert_ne!(native.stderr.len(), 0, "{root:?}");
        assert_eq!(report["plate_index"], 0);
        None
    };
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    let ares_cwd = root.join("ares/cwd");
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_cwd), inventory(&native_cwd), "{root:?}");
    let actual = fs::read(ares_cwd.join("out/result.json")).unwrap();
    assert!(
        result_comparison_bytes(&actual) == result_comparison_bytes(&native_report),
        "complete report differs: {root:?}"
    );
    if let Some(expected) = native_gcode {
        assert!(
            gcode(&ares_cwd) == expected,
            "complete G-code differs: {root:?}"
        );
    }
}

fn compare_multiasingle_toolchange(mode: &str, switches_filament: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-load-custom-gcodes-toolchange-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    write_toolchange_fixture(&root);
    let events = root.join("events.json");
    fs::write(
        &events,
        serde_json::to_vec(&json!({
            "mode": mode,
            "gcodes": [{
                "type": "ToolChange", "print_z": 0.4, "color": "",
                "extruder": 2, "extra": ""
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let argv = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        "1",
        "--load-settings",
        &format!(
            "{};{}",
            root.join("machine.json").display(),
            root.join("process.json").display()
        ),
        "--load-filaments",
        &format!(
            "{};{}",
            root.join("filament-a.json").display(),
            root.join("filament-b.json").display()
        ),
        "--load-custom-gcodes",
        events.to_str().unwrap(),
        root.join("block.stl").to_str().unwrap(),
    ]
    .map(OsString::from);
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_cwd), inventory(&native_cwd), "{root:?}");
    let native_report = fs::read(native_cwd.join("out/result.json")).unwrap();
    let actual = fs::read(ares_cwd.join("out/result.json")).unwrap();
    assert!(
        result_comparison_bytes(&actual) == result_comparison_bytes(&native_report),
        "complete report differs: {root:?}"
    );
    let native_gcode = gcode(&native_cwd);
    let ares_gcode = gcode(&ares_cwd);
    assert_eq!(
        ares_gcode, native_gcode,
        "complete G-code differs: {root:?}"
    );
    let tool_change = b"T1\n";
    let expected_tool_changes = if switches_filament { 1 } else { 0 };
    assert_eq!(
        native_gcode
            .windows(tool_change.len())
            .filter(|window| *window == tool_change)
            .count(),
        expected_tool_changes,
        "{root:?}"
    );
}

fn write_toolchange_fixture(root: &Path) {
    let write = |name: &str, value: Value| {
        fs::write(root.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    write(
        "machine.json",
        json!({
            "type": "machine", "name": "Synthetic dual-filament machine", "from": "system",
            "printer_technology": "FFF", "printable_height": "200",
            "printable_area": ["0x0", "200x0", "200x200", "0x200"],
            "nozzle_diameter": ["0.4"], "printer_extruder_id": ["1"],
            "printer_extruder_variant": ["Direct Drive Standard"],
            "filament_diameter": ["1.75", "1.75"], "layer_change_gcode": "G92 E0"
        }),
    );
    write(
        "process.json",
        json!({
            "type": "process", "name": "Synthetic single-object process", "from": "system",
            "compatible_printers": ["Synthetic dual-filament machine"], "layer_height": "0.2",
            "initial_layer_print_height": "0.2", "sparse_infill_density": "0%",
            "top_shell_layers": "0", "bottom_shell_layers": "0", "enable_prime_tower": "0"
        }),
    );
    for (name, id, density, temperature) in [
        ("filament-a.json", "SYNTH_A", "1.11", "205"),
        ("filament-b.json", "SYNTH_B", "1.33", "225"),
    ] {
        write(
            name,
            json!({
                "type": "filament", "name": name, "from": "system", "filament_id": id,
                "filament_diameter": ["1.75"], "filament_density": [density],
                "filament_is_support": ["0"], "nozzle_temperature": [temperature],
                "filament_extruder_variant": ["Direct Drive Standard"],
                "filament_start_gcode": ["; synthetic filament"]
            }),
        );
    }
    fs::write(
        root.join("block.stl"),
        super::direct_stl_slice::block_stl(108.0),
    )
    .unwrap();
}

fn seed_archived_event(project: &Path) {
    let mut source = ZipArchive::new(Cursor::new(fs::read(project).unwrap())).unwrap();
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        if entry.is_dir() {
            archive.add_directory(entry.name(), options).unwrap();
        } else {
            let project_settings = entry.name() == "Metadata/project_settings.config";
            archive.start_file(entry.name(), options).unwrap();
            if project_settings {
                // Materialize the non-pellet, unrestricted filament defaults in the saved fixture.
                let mut config: Value = serde_json::from_reader(entry).unwrap();
                config["compatible_process_expression_group"] = json!([""]);
                config["pellet_flow_coefficient"] = json!(["0.4157"]);
                serde_json::to_writer(&mut archive, &config).unwrap();
            } else {
                io::copy(&mut entry, &mut archive).unwrap();
            }
        }
    }
    archive
        .start_file(
            "Metadata/custom_gcode_per_layer.xml",
            SimpleFileOptions::default(),
        )
        .unwrap();
    write!(
        archive,
        r#"<?xml version="1.0" encoding="utf-8"?>
<custom_gcodes_per_layer><plate><plate_info id="1"/>
<layer top_z="0.4" type="4" extruder="1" color="" extra="{ARCHIVED}"/>
<mode value="SingleExtruder"/></plate></custom_gcodes_per_layer>"#
    )
    .unwrap();
    fs::write(project, archive.finish().unwrap().into_inner()).unwrap();
}
