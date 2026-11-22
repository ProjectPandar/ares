use serde_json::{Map, Value, json};
use std::{collections::BTreeSet, fs, path::Path};

use super::{OrcaRunner, Stage, run_installed_pair, stages};

const CUBE: &[u8] = include_bytes!("../../../../../tests/parity/cube10.stl");
const MACHINE_PROCESS: &str = "machine.json;process.json";
const REPEATED_PLA: &str = "pla.json;pla.json";
const PRESETS: &[(&str, &[u8])] = &[
    (
        "machine.json",
        include_bytes!("../../../../../tests/parity/bbl-explicit-settings/machine.json"),
    ),
    (
        "process.json",
        include_bytes!("../../../../../tests/parity/bbl-explicit-settings/process.json"),
    ),
    (
        "pla.json",
        include_bytes!("../../../../../tests/parity/bbl-explicit-settings/pla.json"),
    ),
    (
        "petg.json",
        include_bytes!("../../../../../tests/parity/bbl-explicit-settings/petg.json"),
    ),
];
const PLA: &str = "CR-PLA @Ender-3-all";
const PETG: &str = "CR-PETG @Ender-3-all";

#[test]
fn repeated_explicit_filament_disables_the_tower_and_marks_the_process_dirty() {
    compare(Some(Some(REPEATED_PLA)), PLA, "0", "enable_prime_tower");
}

#[test]
fn distinct_explicit_filaments_keep_the_tower_and_replace_the_second_slot() {
    compare(Some(Some("pla.json;petg.json")), PETG, "1", "");
}

#[test]
fn explicit_machine_and_process_keep_the_saved_filament_slots() {
    compare(Some(None), PLA, "1", "");
}

#[test]
fn saved_settings_without_explicit_presets_keep_the_tower_and_both_slots() {
    compare(None, PLA, "1", "");
}

fn export_cube(root: &Path, runner: &OrcaRunner) {
    fs::write(root.join("cube.stl"), CUBE).unwrap();
    for (name, bytes) in PRESETS {
        fs::write(root.join(name), bytes).unwrap();
    }
    stages::run(
        runner.bin(),
        [
            "--export-3mf",
            "input.3mf",
            "--load-settings",
            MACHINE_PROCESS,
            "--load-filaments",
            REPEATED_PLA,
            "cube.stl",
        ]
        .map(str::to_owned),
        Stage::Export,
        root,
    )
    .unwrap_or_else(|error| panic!("{error}; {root:?}"));
}

fn compare(presets: Option<Option<&str>>, second_filament: &str, tower: &str, process_dirty: &str) {
    let mut argv = vec!["--export-settings", "settings.json"];
    if let Some(filaments) = presets {
        argv.extend(["--load-settings", MACHINE_PROCESS]);
        if let Some(filaments) = filaments {
            argv.extend(["--load-filaments", filaments]);
        }
    }
    argv.push("input.3mf");
    let (root, native, ares) =
        run_installed_pair(export_cube, &argv, Stage::Export, "settings.json");
    let native: Map<String, Value> = serde_json::from_slice(&native).unwrap();
    let ares: Map<String, Value> = serde_json::from_slice(&ares).unwrap();
    assert_eq!(
        native["filament_settings_id"],
        json!([PLA, second_filament])
    );
    assert_eq!(native["enable_prime_tower"], tower);
    assert_eq!(
        native["different_settings_to_system"],
        json!([process_dirty, "", "", ""])
    );
    let differences = native
        .keys()
        .chain(ares.keys())
        .filter(|key| native.get(*key) != ares.get(*key))
        .collect::<BTreeSet<_>>();
    assert!(
        differences.is_empty(),
        "different settings {differences:?}; {root:?}"
    );
}
