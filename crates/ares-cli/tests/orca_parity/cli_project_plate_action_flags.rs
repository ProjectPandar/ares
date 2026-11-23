#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::{Value, json};

use crate::{
    golden::compare_ordered_bytes_generator_only,
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, stages},
};

const PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/custom-gcode-overview/two-plates.3mf");
const EVENTS: &str = "Metadata/custom_gcode_per_layer.xml";
const SETTINGS: &str = "Metadata/project_settings.config";
const ALL: &[&str] = &["plate_1.gcode", "plate_2.gcode"];
const FIRST: &[&str] = &["plate_1.gcode"];
const SECOND: &[&str] = &["plate_2.gcode"];

#[test]
fn cli_project_plates_check_embedded_post_processing_unless_disabled() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = root("project-normative-check");
    edit_project(&root, SETTINGS, |text| {
        let mut settings: Value = serde_json::from_str(&text).unwrap();
        settings["post_process"] = json!([""]);
        settings.to_string()
    });
    for (selection, names) in [("0", ALL), ("2", SECOND)] {
        paired(
            &runner,
            &root,
            (&format!("disabled-{selection}"), selection),
            &["--normative-check=0"],
            (0, names, None),
        );
        paired(
            &runner,
            &root,
            (&format!("default-{selection}"), selection),
            &[],
            (237, &[], None),
        );
    }
}

#[test]
fn cli_project_plates_replace_saved_events_before_filtering_custom_events() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = root("project-custom-gcodes");
    edit_project(&root, EVENTS, |_| {
        r#"<?xml version="1.0" encoding="utf-8"?>
<custom_gcodes_per_layer>
  <plate><plate_info id="1"/>
    <layer top_z="0.6" type="1" extruder="1" color="" extra="Saved first pause"/>
    <layer top_z="1" type="4" extruder="1" color="" extra="M117 Saved first event"/>
  </plate>
  <plate><plate_info id="2"/>
    <layer top_z="0.6" type="1" extruder="1" color="" extra="Saved second pause"/>
    <layer top_z="1" type="4" extruder="1" color="" extra="M117 Saved second event"/>
  </plate>
</custom_gcodes_per_layer>"#
            .to_owned()
    });
    fs::write(
        root.join("events.json"),
        json!({
            "mode": "SingleExtruder",
            "gcodes": [
                {"type": "PausePrint", "print_z": 0.6, "extruder": 1, "color": "", "extra": "External pause"},
                {"type": "Custom", "print_z": 1.0, "extruder": 1, "color": "", "extra": "M117 External event"}
            ]
        })
        .to_string(),
    )
    .unwrap();
    let events = root.join("events.json");
    let load = ["--load-custom-gcodes", events.to_str().unwrap()];
    let skip = "--skip-modified-gcodes";
    let actual = paired(
        &runner,
        &root,
        ("loaded-and-filtered", "0"),
        &[skip, load[0], load[1]],
        (0, ALL, None),
    );
    assert!(actual[0].lines().any(|line| line == "M25"));
    assert!(!actual[1].lines().any(|line| line == "M25"));
    assert!(actual.iter().all(|gcode| !gcode.contains("M117 ")));
    let actual = paired(
        &runner,
        &root,
        ("selected-loaded-and-filtered", "2"),
        &[load[0], load[1], skip],
        (0, SECOND, None),
    );
    assert!(actual[0].lines().any(|line| line == "M25"));
    assert!(!actual[0].contains("M117 "));
    let actual = paired(&runner, &root, ("loaded-only", "0"), &load, (0, ALL, None));
    assert!(actual[0].contains("M117 External event"));
    assert!(!actual[1].contains("M117 Saved second event"));
    let actual = paired(
        &runner,
        &root,
        ("filtered-only", "0"),
        &[skip],
        (0, ALL, None),
    );
    for gcode in actual {
        assert!(gcode.lines().any(|line| line == "M25"));
        assert!(!gcode.contains("M117 Saved"));
    }
}

#[test]
fn cli_project_downward_compatibility_intersects_the_imported_plate_bounds() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = root("project-downward-check");
    edit_project(&root, "3D/3dmodel.model", |text| {
        let old = "1 0 0 0 1 0 0 0 1 374 110 5";
        assert_eq!(text.matches(old).count(), 1);
        text.replace(old, "2 0 0 0 1 0 0 0 1 374 110 5")
    });
    let paths = [
        target(&root, "First fit", 25),
        target(&root, "Too narrow for the second plate", 15),
        target(&root, "Second fit", 30),
    ];
    let list = paths.join(";");
    let selected = json!(["First fit", "Too narrow for the second plate", "Second fit"]);
    let all = json!(["First fit", "Second fit"]);
    for (selection, names, expected) in [("0", ALL, &all), ("1", FIRST, &selected)] {
        paired(
            &runner,
            &root,
            (&format!("all-targets-{selection}"), selection),
            &["--downward-check", "--downward-settings", &list],
            (0, names, Some(expected)),
        );
    }
    paired(
        &runner,
        &root,
        ("no-fit", "2"),
        &["--downward-check=1", "--downward-settings", &paths[1]],
        (0, SECOND, None),
    );
    let missing = root.join("missing.json");
    let settings = ["--downward-settings", missing.to_str().unwrap()];
    paired(
        &runner,
        &root,
        ("settings-only", "1"),
        &settings,
        (0, FIRST, None),
    );
    paired(
        &runner,
        &root,
        ("missing-target", "0"),
        &["--downward-check", settings[0], settings[1]],
        (253, &[], None),
    );
}

fn root(prefix: &str) -> PathBuf {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep()
}

fn edit_project(root: &Path, path: &str, edit: impl FnOnce(String) -> String) {
    let mut input = zip::ZipArchive::new(Cursor::new(PROJECT)).unwrap();
    let mut output = zip::ZipWriter::new(fs::File::create(root.join("project.3mf")).unwrap());
    let mut edit = Some(edit);
    for index in 0..input.len() {
        let mut entry = input.by_index(index).unwrap();
        if entry.name() == path {
            let mut text = String::new();
            entry.read_to_string(&mut text).unwrap();
            output
                .start_file(path, zip::write::SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(edit.take().unwrap()(text).as_bytes())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    assert!(edit.is_none(), "missing {path}");
    output.finish().unwrap();
}

fn target(root: &Path, name: &str, width: i32) -> String {
    let path = root.join(format!("{width}.json"));
    fs::write(
        &path,
        json!({
            "name": name, "type": "machine", "from": "system",
            "printable_area": ["0x0", format!("{width}x0"), format!("{width}x30"), "0x30"],
            "printable_height": "30", "bed_exclude_area": [], "wrapping_exclude_area": []
        })
        .to_string(),
    )
    .unwrap();
    path.into_os_string().into_string().unwrap()
}

fn paired(
    runner: &OrcaRunner,
    root: &Path,
    (label, selection): (&str, &str),
    flags: &[&str],
    (exit, names, downward): (i32, &[&str], Option<&Value>),
) -> Vec<String> {
    let input = root.join("project.3mf");
    let args = [
        ["--slice", selection, "--outputdir", "out"].as_slice(),
        flags,
        &[input.to_str().unwrap()],
    ]
    .concat();
    let native = root.join(format!("{label}-native"));
    let ares = root.join(format!("{label}-ares"));
    for work in [&native, &ares] {
        fs::create_dir_all(work.join("out")).unwrap();
        fs::write(work.join("argv.json"), serde_json::to_vec(&args).unwrap()).unwrap();
    }
    let result = stages::run_live(
        runner.bin(),
        args.iter().map(|arg| (*arg).to_owned()),
        stages::Stage::Slice,
        &native,
    );
    let code = result.map_or_else(|error| error.captured.unwrap().code, |()| Some(0));
    assert_eq!(code, Some(exit), "Native: {native:?}");
    let expected = outputs(&native, names, downward);
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(&ares)
        .args(&args)
        .output()
        .unwrap();
    fs::write(ares.join("stdout"), &output.stdout).unwrap();
    fs::write(ares.join("stderr"), &output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        code,
        "{ares:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = outputs(&ares, names, downward);
    assert_eq!(expected.1, actual.1, "result.json: {label}");
    for (expected, actual) in expected.0.iter().zip(&actual.0) {
        compare_ordered_bytes_generator_only(expected.as_bytes(), actual.as_bytes())
            .unwrap_or_else(|error| panic!("{label}: {error}"));
    }
    actual.0
}

fn outputs(work: &Path, names: &[&str], downward: Option<&Value>) -> (Vec<String>, Vec<u8>) {
    let directory = work.join("out");
    let mut actual_names = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    actual_names.sort();
    assert_eq!(
        actual_names,
        [names, &["result.json"]].concat(),
        "{directory:?}"
    );
    let report = fs::read(directory.join("result.json")).unwrap();
    let document: Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(
        document.get("downward_compatible_machine"),
        downward,
        "{directory:?}"
    );
    (
        names
            .iter()
            .map(|name| fs::read_to_string(directory.join(name)).unwrap())
            .collect(),
        result_comparison_bytes(&report),
    )
}
