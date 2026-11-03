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
use crate::{self as parity, CaseInputs, smoke_overrides};
use serde_json::{Value, json};

fn dual_extruder_case(
    runner: &OrcaRunner,
    label: &str,
    filaments: Vec<serde_json::Map<String, Value>>,
) -> crate::runner::ExportedCase {
    let profiles = crate::presets::VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "BBL",
    )
    .unwrap();
    let selection = parity::select_printer(&profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), Value::String("Manual".into()));
    machine.insert(
        "curr_bed_type".into(),
        Value::String("High Temp Plate".into()),
    );
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
) -> serde_json::Map<String, Value> {
    let selection = parity::select_printer(profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut filament = profiles.filament(&selection.filaments[0]).unwrap();
    filament.insert("filament_colour".into(), json!([colour]));
    filament.insert("filament_map".into(), json!([map]));
    filament
}

#[test]
fn filament_map_cross_map_matches_native() {
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
            settings["filament_map"],
            json!(maps),
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
    let sliced = runner.slice_case(&out_of_range);
    // `ToolOrdering.cpp:990` indexes `extruder_count` past its length for
    // an out-of-range map, so the native reference crashes before
    // producing a stream. The crash is recorded as the upstream fact —
    // it never serves as a comparison reference.
    assert!(
        sliced.is_err(),
        "the out-of-range map must fail the native reference (upstream SIGSEGV, ToolOrdering.cpp:990)"
    );
    // Ares rejects the same project through the typed option boundary
    // (`resolve_filament_indices` bounds-checks the mapped slot).
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let Err(error) = runtime.block_on(crate::runner::application::slice_project(
        &out_of_range.project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )) else {
        panic!("the out-of-range map must be rejected by the typed option boundary");
    };
    assert!(
        error.to_string().contains("filament_map"),
        "expected the filament_map typed rejection, got {error}"
    );
}

/// SEMM ramming coverage (`WipeTower2.cpp:1378-1390`, `if (m_semm)`):
/// two objects on filaments 1/2 force a tool change under the prime
/// tower, and each filament's `filament_ramming_parameters` profile
/// shapes its own tool-change block. Confirmed against native first
/// (`native_ramming_blocks`), then compared with Ares.
#[test]
fn semm_ramming_profile_changes_tool_change_block() {
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
    let mut machine = profiles.machine(&selection.printer).unwrap();
    // Two filaments on a single-extruder machine: the per-filament machine
    // lists must match the filament count or the native export rejects the
    // project ("filament_is_support's count 1 not equal to filament_colour's
    // size 2").
    machine.insert("filament_is_support".into(), json!(["0", "0"]));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), Value::String("1".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    // The Ender-3 bed is 220×220 and its profile carries no tower position;
    // the BBL-flavored default lands outside the bed and the native export
    // rejects the project. 150/150 keeps the tower and its purge moves
    // inside the printable area (the generated tool-change purges reach
    // ~66mm past the anchor).
    process.insert("wipe_tower_x".into(), Value::String("150".into()));
    process.insert("wipe_tower_y".into(), Value::String("150".into()));
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
    let model = repo_root().join("tests/parity/cube10.stl");
    let exported = runner
        .export_case_multi(
            &CaseInputs {
                label: "semm-ramming/profile-variant",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &[&model, &model],
            &[1, 2],
        )
        .unwrap();
    let settings = application::settings(&exported.project).unwrap();
    assert_eq!(settings["filament_colour"], json!(["#F2754E", "#0000FF"]));
    let sliced = runner.slice_case(&exported).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(
        native.contains("\nM600\n") || native.contains("\nT1\n"),
        "the SEMM fixture must contain a tool change for the ramming path to fire"
    );
    let blocks = native_ramming_blocks(native);
    assert!(
        blocks.len() >= 2,
        "native must emit tool-change blocks for both filaments"
    );
    // Swap the ramming profiles between the filaments and require the first
    // tool-change block to change — the per-filament profile feeds the
    // ramming moves.
    let swapped_filaments = vec![
        ramming_filament(short_profile, "#F2754E"),
        ramming_filament(default_profile, "#0000FF"),
    ];
    let swapped = runner
        .export_case_multi(
            &CaseInputs {
                label: "semm-ramming/profile-swapped",
                machine: &machine,
                process: &process,
                filaments: &swapped_filaments,
            },
            &smoke_overrides(),
            &[&model, &model],
            &[1, 2],
        )
        .unwrap();
    let swapped_sliced = runner.slice_case(&swapped).unwrap();
    let swapped_native = std::str::from_utf8(&swapped_sliced.reference).unwrap();
    let swapped_blocks = native_ramming_blocks(swapped_native);
    assert!(
        swapped_blocks.len() >= 2,
        "the swapped fixture must keep its tool changes"
    );
    assert_ne!(
        blocks[0], swapped_blocks[0],
        "the ramming profile must shape the tool-change block"
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

/// The tool-change blocks: from each `M600` (change_filament_gcode) to
/// the next, covering the unload/ramming/load sequence where
/// WipeTower2.cpp:1378-1390 emits the ramming moves.
fn native_ramming_blocks(gcode: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut rest = gcode;
    while let Some(position) = rest.find("\nM600\n") {
        let start = &rest[position + 1..];
        match start["M600\n".len()..].find("\nM600\n") {
            Some(next) => {
                let end = next + 1 + "M600\n".len();
                blocks.push(&start[..end]);
                rest = &start[end..];
            }
            None => {
                blocks.push(start);
                break;
            }
        }
    }
    blocks
}
