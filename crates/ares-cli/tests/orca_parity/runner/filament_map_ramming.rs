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
    let Err(error) = runtime.block_on(ares_core::slice_project(
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
