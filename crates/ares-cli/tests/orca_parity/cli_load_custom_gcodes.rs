#![cfg(target_os = "linux")]

#[path = "cli_load_custom_gcodes/toolchange.rs"]
mod toolchange;

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
    toolchange::compare_multiasingle_toolchange("MultiAsSingle", false, &[(0.4, 2)], &[0, 1]);
}

#[test]
fn cli_load_custom_gcodes_single_filament_toolchange_keeps_object_filament() {
    toolchange::compare_multiasingle_toolchange("MultiExtruder", true, &[(0.4, 2)], &[0]);
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
