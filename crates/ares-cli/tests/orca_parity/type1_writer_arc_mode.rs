use std::{
    fs,
    io::{Cursor, Read as _, Write as _},
};

use serde_json::{Map, Value, json};

use crate::{
    CaseInputs, OrcaRunner, golden,
    runner::{
        application,
        native_api_cli::{NativeCase, sha},
        native_stream_case,
    },
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

fn assert_native_mode(case: &NativeCase, mode: &str) {
    for line in [
        "; wipe_tower_type = type1".to_owned(),
        format!("; enable_arc_fitting = {mode}"),
    ] {
        assert!(
            case.native.lines().any(|actual| actual == line),
            "{line}: {:?}",
            case.root
        );
    }
    for tool in ["T0", "T1"] {
        assert!(
            case.native.lines().any(|line| line.trim_end() == tool),
            "{tool}: {:?}",
            case.root
        );
    }
}

fn first_tower(gcode: &[u8], qidi_progress: bool) -> Vec<u8> {
    let text = std::str::from_utf8(gcode).unwrap();
    let (prefix, _) = text.split_once("; WIPE_TOWER_END\n").unwrap();
    let (approach, tower) = prefix.split_once("; WIPE_TOWER_START\n").unwrap();
    // The Qidi progress marker can land on either side of the same brim move.
    [approach, "; WIPE_TOWER_START\n"]
        .into_iter()
        .chain(
            tower
                .split_inclusive('\n')
                .filter(|line| !(qidi_progress && line.starts_with("M73 "))),
        )
        .chain(["; WIPE_TOWER_END\n"])
        .collect::<String>()
        .into_bytes()
}

fn compare_disabled(runner: &OrcaRunner, label: &str, project: &[u8], qidi_progress: bool) {
    let reference = NativeCase::slice(runner, project, &format!("type1-writer-arc-mode-{label}-"));
    let replay = NativeCase::slice(
        runner,
        project,
        &format!("type1-writer-arc-mode-{label}-replay-"),
    );
    assert_native_mode(&reference, "disabled");
    assert_native_mode(&replay, "disabled");
    let root = &reference.root;
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    let output = root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI: {root:?}");
    let cli = fs::read(output).unwrap();
    let native = reference.native.as_bytes();
    let whole = |gcode| golden::compare_ordered_bytes_generator_only(native, gcode);
    let span = first_tower(native, qidi_progress);
    let scoped = |gcode| {
        golden::compare_ordered_bytes_generator_only(&span, &first_tower(gcode, qidi_progress))
    };
    let scopes = [scoped(replay.native.as_bytes()), scoped(&api), scoped(&cli)];
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "case": label, "input_sha256": sha(project), "upstream": UPSTREAM,
        "replay_root": replay.root,
        "whole_native_replay": whole(replay.native.as_bytes()),
        "whole_native_api": whole(&api), "whole_native_cli": whole(&cli),
        "scope": "through the first WIPE_TOWER_END, inclusive",
        "excludes_qidi_progress": qidi_progress,
        "scoped_native_replay": scopes[0], "scoped_native_api": scopes[1], "scoped_native_cli": scopes[2],
        "api_cli_equal": api == cli,
    })).unwrap()).unwrap();
    for (adapter, result) in ["Native replay", "API", "CLI"].into_iter().zip(scopes) {
        result.unwrap_or_else(|error| panic!("{adapter} first tower at {root:?}: {error}"));
    }
    assert!(api == cli, "API/CLI raw equality: {root:?}");
}

#[test]
fn disabled_semm_rib_and_brim() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let bytes = include_bytes!("../../../../tests/parity/semm-ramming/R1.3mf");
    let project = with_settings(bytes, |config| {
        assert_eq!(config["wipe_tower_type"], "type2");
        config["wipe_tower_type"] = json!("type1");
    });
    compare_disabled(&runner, "semm-rib-and-brim", &project, false);
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
    compare_disabled(&runner, "stock-qidi", &project, true);
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
    native_stream_case::check(
        ("type1-writer-arc-mode", UPSTREAM),
        "enabled-control",
        &project,
        &sha(&project),
        (true, |reference, _| {
            assert_native_mode(reference, "enabled");
            let first = first_tower(reference.native.as_bytes(), false);
            let (_, tower) = std::str::from_utf8(&first)
                .unwrap()
                .split_once("; WIPE_TOWER_START\n")
                .unwrap();
            assert!(
                tower
                    .lines()
                    .any(|line| line.starts_with("G2 ") || line.starts_with("G3 "))
            );
        }),
    );
}
