use std::{
    fs,
    io::{Cursor, Read as _, Write as _},
};

use serde_json::{Map, Value, json};

use crate::{
    CaseInputs, OrcaRunner,
    runner::{native_api_cli::sha, native_stream_case},
};

const UPSTREAM: &[&str] = &[
    "WipeTower.cpp:1122–1130,1382–1388,2017,3817,5382–5385",
    "Polyline.cpp:155–165",
];

fn with_settings(project: &[u8], edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut target = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    let mut edit = Some(edit);
    for i in 0..source.len() {
        let mut entry = source.by_index(i).unwrap();
        if entry.is_dir() {
            target.add_directory(entry.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings = serde_json::from_slice(&bytes).unwrap();
            edit.take().unwrap()(&mut settings);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        target.start_file(entry.name(), options).unwrap();
        target.write_all(&bytes).unwrap();
    }
    assert!(edit.is_none(), "fixture must have project settings");
    target.finish().unwrap().into_inner()
}

fn compare(label: &str, project: &[u8], mode: &str) {
    native_stream_case::check(
        ("type1-writer-arc-mode", UPSTREAM),
        label,
        project,
        &sha(project),
        (true, |reference, _| {
            for line in [
                "; wipe_tower_type = type1".to_owned(),
                format!("; enable_arc_fitting = {mode}"),
            ] {
                assert!(
                    reference.native.lines().any(|actual| actual == line),
                    "{line}: {:?}",
                    reference.root
                );
            }
            let tower = reference.native.split("; WIPE_TOWER_START\n").skip(1);
            let arcs = tower
                .flat_map(|block| block.split_once("; WIPE_TOWER_END\n").unwrap().0.lines())
                .filter(|line| line.starts_with("G2 ") || line.starts_with("G3 "))
                .count();
            assert!(reference.native.contains("; WIPE_TOWER_START\n"));
            if mode == "enabled" {
                assert!(arcs > 0, "Native enabled tower must emit arcs");
            }
        }),
    );
}

#[test]
fn disabled_semm_rib_and_brim() {
    if OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let bytes = include_bytes!("../../../../tests/parity/semm-ramming/R1.3mf");
    let project = with_settings(bytes, |config| {
        assert_eq!(config["wipe_tower_type"], "type2");
        config["wipe_tower_type"] = json!("type1");
    });
    compare("semm-rib-and-brim", &project, "disabled");
}

#[test]
fn disabled_stock_qidi_tower_entry() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = crate::presets::VendorProfiles::load(
        &crate::runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Qidi",
    )
    .unwrap();
    let machine = profiles.machine("Qidi Q2 0.4 nozzle").unwrap();
    let process = profiles.process("0.20mm Standard @Qidi Q2").unwrap();
    let mut filament = profiles
        .filament("QIDI PLA Rapido @Qidi Q2 0.4 nozzle")
        .unwrap();
    filament
        .entry("filament_is_support")
        .or_insert_with(|| json!(["0"]));
    let models = tempfile::tempdir().unwrap();
    let cube = crate::runner::repo_root().join("tests/parity/cube10.stl");
    let second_cube = models.path().join("r4-second-cube10.stl");
    fs::copy(&cube, &second_cube).unwrap();
    let exported = runner
        .export_case_multi(
            &CaseInputs {
                label: "type1-writer-arc-mode-qidi",
                machine: &machine,
                process: &process,
                filaments: &[filament.clone(), filament],
            },
            &Map::new(),
            &[&cube, &second_cube],
            &[1, 2],
        )
        .unwrap();
    let project = with_settings(&exported.project, |config| {
        config["filament_colour"] = json!(["#0060C0", "#FDBD10"]);
    });
    compare("stock-qidi", &project, "disabled");
}

#[test]
fn enabled_type1_arc_control() {
    if OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let bytes = include_bytes!("../../../../tests/parity/bbl-type1-tower/enabled.3mf");
    let project = with_settings(bytes, |config| {
        config["wipe_tower_type"] = json!("type1");
    });
    compare("enabled-control", &project, "enabled");
}
