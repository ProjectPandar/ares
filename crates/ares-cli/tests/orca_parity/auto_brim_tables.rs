//! Brim.cpp:260-305,448-530; Model.cpp:3200-3280,3326: group/material witnesses.

use crate::{
    OrcaRunner,
    runner::{
        native_api_cli::{NativeCase, sha},
        volume_fixture,
    },
};

const MULTI: &[u8] =
    include_bytes!("../../../../tests/parity/per-object-skirt-brim-types/auto-multisland.3mf");
const MULTI_SHA: &str = "d3f23cea67a0e0137dd69f7a7a1d898ffe059fe5521af5c2f7b2e768bdc73eae";
const PETG: &[u8] =
    include_bytes!("../../../../tests/parity/per-object-skirt-brim-types/auto-petg.3mf");
const PETG_SHA: &str = "a92a043aa16d538e8732ab389849130f7d5ba39cb5c9ba4a1ce781eb6e55d7c1";

#[test]
fn orca_parity_auto_brim_multisland_width() {
    run_case("auto-brim-multisland-", MULTI, MULTI_SHA, true);
}

#[test]
fn orca_parity_auto_brim_petg_tables() {
    run_case("auto-brim-petg-", PETG, PETG_SHA, false);
}

#[test]
fn orca_parity_auto_brim_derived_small_area_bump() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = volume_fixture::add_volumes(MULTI, "normal_part", &[(0.35, 0.0)], &[]);
    run_derived_case(&runner, &project, "auto-brim-derived-bump-", true);
}

#[test]
fn orca_parity_auto_brim_derived_multi_material() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let base = NativeCase::slice(&runner, MULTI, "auto-brim-derived-base-");
    assert!(base.native.contains(";TYPE:Brim\n"));
    let project = volume_fixture::add_volumes(
        MULTI,
        "normal_part",
        &[(0.3, 5.0)],
        &[("extruder", "2"), ("filament_type", "PETG")],
    );
    let changed = NativeCase::slice(&runner, &project, "auto-brim-derived-multi-material-");
    assert_ne!(
        brim_signatures(&base.native),
        brim_signatures(&changed.native)
    );
    run_derived_case(
        &runner,
        &project,
        "auto-brim-derived-multi-material-",
        false,
    );
}

fn run_derived_case(runner: &OrcaRunner, project: &[u8], prefix: &str, require_brim: bool) {
    let case = NativeCase::slice(runner, project, prefix);
    let mut in_brim = false;
    let mut brim_lines = 0;
    for line in case.native.lines() {
        if line == ";TYPE:Brim" {
            in_brim = true;
        } else if line.starts_with(";TYPE:") || line == ";LAYER_CHANGE" {
            in_brim = false;
        } else if in_brim && line.starts_with("G1 ") {
            brim_lines += 1;
        }
    }
    if require_brim {
        assert!(
            case.native.contains(";TYPE:Brim\n"),
            "{prefix} Native did not activate Brim"
        );
        assert!(
            brim_lines > 0,
            "{prefix} Native Brim has no body-coordinate moves"
        );
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 18, 0, 0, 0,
    ));
}

fn run_case(prefix: &str, project: &[u8], expected_sha: &str, multisland: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, prefix);
    assert!(case.native.contains(";TYPE:Brim\n"));
    if multisland {
        let signatures = brim_signatures(&case.native);
        assert!(signatures.len() >= 2, "expected two Brim blocks");
        assert_ne!(signatures[0], signatures[1], "islands must differ");
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 17, 0, 0, 0,
    ));
}

fn brim_signatures(gcode: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut current = None;
    for line in gcode.lines() {
        if line == ";TYPE:Brim" {
            current = Some((0usize, String::new()));
        } else if line.starts_with(";TYPE:") || line == ";LAYER_CHANGE" {
            if let Some(block) = current.take() {
                blocks.push(block);
            }
        } else if let Some((count, signature)) = &mut current
            && line.starts_with("G1 ")
        {
            *count += 1;
            signature.push_str(line);
            signature.push('\n');
        }
    }
    if let Some(block) = current {
        blocks.push(block);
    }
    blocks
}
