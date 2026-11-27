//! Whole-G-code ordinary catalog-backed retained-filament checks
//! (`OrcaSlicer.cpp:2775–2815,3510–3811`, skip branch `:3670–3681`).
use std::fs;

use serde_json::json;

use super::{
    gcodes_support::{
        COLUMNS, SKIP, columns, config, decode_strings, pair_on, repack_file, same, stock_resources,
    },
    {OrcaRunner, artifact_dir},
};
use crate::runner::repo_root;

const SAVED_START: &str = "M117 SAVED CATALOG FILAMENT\n";
const SAVED_SLOT2: &str = "M117 SAVED CATALOG SLOT 2\n";
const SAVED_END_SLOT2: &str = "M117 SAVED CATALOG END 2\n";
const START: &str = "filament_start_gcode";
const END: &str = "filament_end_gcode";

#[test]
fn catalog_retained_filament_dirty_column_omitted_flag_control() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    let base =
        fs::read(repo_root().join("tests/parity/adaptive-cubic-bodies/x1c-stock.3mf")).unwrap();
    repack_file(&base, &root.join("project.3mf"), |saved| {
        saved[START] = json!([SAVED_START]);
        saved["filament_density"] = json!(["1.31"]);
        saved[COLUMNS] = json!(["", "filament_density;filament_start_gcode", ""]);
    });
    let resources = stock_resources();
    let ares_only = ["--resources-dir", resources.to_str().unwrap()];
    let gcode = pair_on((&runner, &root), "project.3mf", "omitted", &[], &ares_only);
    assert_eq!(decode_strings(&gcode, START), [SAVED_START]);
    assert_eq!(
        columns(&gcode),
        ["", "filament_density;filament_start_gcode", ""]
    );
    assert_eq!(config(&gcode, "filament_density"), "1.31");
}

#[test]
fn catalog_retained_filament_dirty_column_resets_script_only() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    let base =
        fs::read(repo_root().join("tests/parity/adaptive-cubic-bodies/x1c-stock.3mf")).unwrap();
    repack_file(&base, &root.join("project.3mf"), |saved| {
        saved[START] = json!([SAVED_START]);
        saved["filament_density"] = json!(["1.31"]);
        saved[COLUMNS] = json!(["", "filament_density;filament_start_gcode", ""]);
    });
    let resources = stock_resources();
    let ares_only = ["--resources-dir", resources.to_str().unwrap()];
    let omitted = pair_on((&runner, &root), "project.3mf", "omitted", &[], &ares_only);
    let off = pair_on(
        (&runner, &root),
        "project.3mf",
        "false",
        &["--skip-modified-gcodes=false"],
        &ares_only,
    );
    same(&omitted, &off, &root);

    let enabled = pair_on(
        (&runner, &root),
        "project.3mf",
        "enabled",
        &[SKIP],
        &ares_only,
    );
    let reset_start = decode_strings(&enabled, START);
    assert_ne!(reset_start, [SAVED_START]);
    assert!(reset_start[0].contains("; filament start gcode"));
    assert_eq!(columns(&enabled), ["", "filament_density", ""]);
    assert_eq!(config(&enabled, "filament_density"), "1.31");
}

#[test]
fn catalog_retained_filament_keep_project_omitted_flag_control() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    let base =
        fs::read(repo_root().join("tests/parity/adaptive-cubic-bodies/x1c-stock.3mf")).unwrap();
    repack_file(&base, &root.join("project.3mf"), |saved| {
        saved[START] = json!([SAVED_START]);
        saved["filament_density"] = json!(["1.31"]);
        saved[COLUMNS] = json!(["", "", ""]);
    });
    let resources = stock_resources();
    let ares_only = ["--resources-dir", resources.to_str().unwrap()];
    let gcode = pair_on((&runner, &root), "project.3mf", "omitted", &[], &ares_only);
    assert_eq!(decode_strings(&gcode, START), [SAVED_START]);
    assert_eq!(columns(&gcode), ["", "", ""]);
    assert_eq!(config(&gcode, "filament_density"), "1.31");
}

#[test]
fn catalog_retained_filament_keep_project_resets_script_without_writing_column() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    let base =
        fs::read(repo_root().join("tests/parity/adaptive-cubic-bodies/x1c-stock.3mf")).unwrap();
    repack_file(&base, &root.join("project.3mf"), |saved| {
        saved[START] = json!([SAVED_START]);
        saved["filament_density"] = json!(["1.31"]);
        saved[COLUMNS] = json!(["", "", ""]);
    });
    let resources = stock_resources();
    let ares_only = ["--resources-dir", resources.to_str().unwrap()];
    let omitted = pair_on((&runner, &root), "project.3mf", "omitted", &[], &ares_only);
    assert_eq!(decode_strings(&omitted, START), [SAVED_START]);
    assert_eq!(columns(&omitted), ["", "", ""]);

    let enabled = pair_on(
        (&runner, &root),
        "project.3mf",
        "enabled",
        &[SKIP],
        &ares_only,
    );
    let reset_start = decode_strings(&enabled, START);
    assert_ne!(reset_start, [SAVED_START]);
    assert!(reset_start[0].contains("; filament start gcode"));
    // keep_project must NOT be serialized back into different_settings_to_system.
    assert_eq!(columns(&enabled), ["", "", ""]);
    assert_eq!(config(&enabled, "filament_density"), "1.31");
}

#[test]
fn catalog_retained_filament_refresh_replaces_empty_notes_vector() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    let base =
        fs::read(repo_root().join("tests/parity/bbl-layer-object-labels/one-object.3mf")).unwrap();
    repack_file(&base, &root.join("two-slot.3mf"), |saved| {
        saved["filament_notes"] = json!("");
        saved[COLUMNS] = json!(["layer_height", "", "", ""]);
    });
    let resources = stock_resources();
    let gcode = pair_on(
        (&runner, &root),
        "two-slot.3mf",
        "empty-notes",
        &[],
        &["--resources-dir", resources.to_str().unwrap()],
    );
    assert_eq!(decode_strings(&gcode, "filament_notes"), ["", ""]);
}

#[test]
fn catalog_retained_multi_filament_preserves_other_slots() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let base =
        fs::read(repo_root().join("tests/parity/bbl-layer-object-labels/one-object.3mf")).unwrap();
    let resources = stock_resources();
    let ares_only = ["--resources-dir", resources.to_str().unwrap()];

    // Case 1: slot 0 resolvable, slot 1 unresolvable — both dirty and keep_project.
    for (label, dirty, expected_dirty) in [
        (
            "dirty",
            [
                "",
                "filament_density;filament_start_gcode",
                "filament_density;filament_start_gcode",
                "",
            ],
            [
                "",
                "filament_density",
                "filament_density;filament_start_gcode",
                "",
            ],
        ),
        ("keep-project", ["", "", "", ""], ["", "", "", ""]),
    ] {
        let root = artifact_dir();
        repack_file(&base, &root.join("two-slot.3mf"), |saved| {
            saved["filament_settings_id"] =
                json!(["Bambu PLA Basic @BBL X1C", "Custom Unresolved PLA"]);
            saved["filament_density"] = json!(["1.31", "1.32"]);
            saved[START] = json!([SAVED_START, SAVED_SLOT2]);
            saved[COLUMNS] = json!(dirty);
        });
        let omitted = pair_on(
            (&runner, &root),
            "two-slot.3mf",
            &format!("omitted-{label}"),
            &[],
            &ares_only,
        );
        assert_eq!(decode_strings(&omitted, START), [SAVED_START, SAVED_SLOT2]);
        assert_eq!(columns(&omitted), dirty);

        let enabled = pair_on(
            (&runner, &root),
            "two-slot.3mf",
            &format!("enabled-{label}"),
            &[SKIP],
            &ares_only,
        );
        let starts = decode_strings(&enabled, START);
        assert_ne!(starts[0], SAVED_START);
        assert!(starts[0].contains("; filament start gcode"));
        assert_eq!(starts[1], SAVED_SLOT2);
        assert_eq!(columns(&enabled), expected_dirty);
        assert_eq!(config(&enabled, "filament_density"), "1.31,1.32");
    }

    // Case 2: both slots resolvable with distinct declared G-code overrides.
    let root = artifact_dir();
    repack_file(&base, &root.join("two-slot.3mf"), |saved| {
        saved["filament_density"] = json!(["1.31", "1.32"]);
        saved[START] = json!([SAVED_START, SAVED_SLOT2]);
        saved[END] = json!(["; filament end gcode \n\n", SAVED_END_SLOT2]);
        saved[COLUMNS] = json!([
            "",
            "filament_density;filament_start_gcode",
            "filament_density;filament_end_gcode;filament_start_gcode",
            ""
        ]);
    });
    let enabled = pair_on(
        (&runner, &root),
        "two-slot.3mf",
        "both-enabled",
        &[SKIP],
        &ares_only,
    );
    let starts = decode_strings(&enabled, START);
    let ends = decode_strings(&enabled, END);
    assert_ne!(starts[0], SAVED_START);
    assert_ne!(starts[1], SAVED_SLOT2);
    assert_ne!(ends[1], SAVED_END_SLOT2);
    assert_eq!(
        columns(&enabled),
        ["", "filament_density", "filament_density", ""]
    );
    assert_eq!(config(&enabled, "filament_density"), "1.31,1.32");
}
