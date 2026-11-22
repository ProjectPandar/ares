use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

use super::{Stage, run_installed_pair};

const PROJECT: &[u8] = include_bytes!("../../../../../tests/parity/bbl-explicit-settings/cube.3mf");
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
    compare(
        Some(Some("pla.json;pla.json")),
        PLA,
        "0",
        "enable_prime_tower",
    );
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

fn compare(presets: Option<Option<&str>>, second_filament: &str, tower: &str, process_dirty: &str) {
    let mut argv = vec!["--export-settings", "settings.json"];
    if let Some(filaments) = presets {
        argv.extend(["--load-settings", "machine.json;process.json"]);
        if let Some(filaments) = filaments {
            argv.extend(["--load-filaments", filaments]);
        }
    }
    argv.push("input.3mf");
    let (root, native, ares) =
        run_installed_pair(PROJECT, PRESETS, &argv, Stage::Export, "settings.json");
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
