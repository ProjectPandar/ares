//! Dedicated runner E2E for the two zero-case inventory plans
//! (`filament-map-ramming-coverage-cases.md`): the sweep's
//! single-filament/single-extruder fixture cannot express either legal
//! domain.
//!
//! `filament_map` (PrintConfig.cpp:2489-2494, coInts default {1}): the
//! cross-map cases need the H2D dual-extruder Manual fixture.
//!
//! `filament_ramming_parameters` (PrintConfig.cpp:2833-2840, coStrings):
//! consumed only through the SEMM ramming path
//! (`WipeTower2.cpp:1378-1390`, `if (m_semm)`), which fires at
//! multi-filament tool changes with the wipe tower.

use crate::runner::{OrcaRunner, application, repo_root};
use crate::{parity, smoke_overrides};
use serde_json::{Value, json};
use std::io::{Cursor, Read as _, Write as _};

fn dual_extruder_case(
    runner: &OrcaRunner,
    label: &str,
    filaments: Vec<Value>,
) -> crate::runner::ExportedCase {
    let profiles = crate::presets::VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "BBL",
    )
    .unwrap();
    let selection = parity::select_printer(&profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), Value::String("Manual".into()));
    machine.insert("curr_bed_type".into(), Value::String("High Temp Plate".into()));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    let exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    runner
        .bind_filament_map_mode(exported, &Value::String("Manual".into()))
        .unwrap()
}

fn h2d_filament(
    profiles: &crate::presets::VendorProfiles,
    colour: &str,
    map: &str,
) -> Value {
    let selection = parity::select_printer(profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut filament = profiles.filament(&selection.filaments[0]).unwrap();
    filament.insert("filament_colour".into(), json!([colour]));
    filament.insert("filament_map".into(), json!([map]));
    filament
}

#[test]
fn orca_parity_filament_map_cross_map_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = crate::presets::VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "BBL",
    )
    .unwrap();
    for (label, maps) in [
        ("filament-map/cross-map/2-1", vec!["2", "1"]),
        ("filament-map/cross-map/1-2", vec!["1", "2"]),
    ] {
        let filaments = maps
            .iter()
            .zip(["#F2754E", "#0000FF"])
            .map(|(map, colour)| h2d_filament(&profiles, colour, map))
            .collect::<Vec<_>>();
        let exported = dual_extruder_case(&runner, label, filaments);
        let settings = application::settings(&exported.project).unwrap();
        assert_eq!(
            settings["filament_map"].as_array().unwrap(),
            &json!(maps),
            "the cross map must reach the exported config"
        );
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", label, outcome.detail);
    }
    let out_of_range = dual_extruder_case(
        &runner,
        "filament-map/out-of-range/3-1",
        vec![
            h2d_filament(&profiles, "#F2754E", "3"),
            h2d_filament(&profiles, "#0000FF", "1"),
        ],
    );
    let outcome = parity::compare_case(&out_of_range);
    assert_eq!(
        outcome.status, "REJECTED",
        "out-of-range extruder index must be rejected: {}",
        outcome.detail
    );
}

#[test]
fn orca_parity_semm_ramming_profile_changes_tool_change_block() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = crate::presets::VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    // SEMM ramming fires at multi-filament tool changes with the wipe tower
    // (`WipeTower2.cpp:1378-1390` gated on `m_semm`).
    process.insert("enable_prime_tower".into(), Value::String("1".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    let default_profile = "120 100 6.6 6.8 7.2 7.6 7.9 8.2 8.7 9.4 9.9 10.0| 0.05 6.6 0.45 6.8 0.95 7.8 1.45 8.3 1.95 9.7 2.45 10 2.95 7.6 3.45 7.6 3.95 7.6 4.45 7.6 4.95 7.6";
    let short_profile = "100 100 1 1|0.05 1 1";
    let ramming_filament = |profile: &str, colour: &str| {
        let mut filament = profiles.filament(&selection.filaments[0]).unwrap();
        filament.insert("filament_colour".into(), json!([colour]));
        filament.insert("filament_ramming_parameters".into(), json!([profile]));
        filament
    };
    let filaments = vec![
        ramming_filament(default_profile, "#F2754E"),
        ramming_filament(short_profile, "#0000FF"),
    ];
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "semm-ramming/profile-variant",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(
        native.contains("\nM620\n") || native.contains("\nT1\n"),
        "the SEMM fixture must contain a tool change for the ramming path to fire"
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
