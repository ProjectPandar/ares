//! Brim.cpp:260-305,448-530; Model.cpp:3258-3280: combined AutoBrim witnesses.

use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn orca_parity_auto_brim_combined_red() {
    run(
        include_bytes!("../../../../tests/parity/per-object-skirt-brim-types/auto-combined.3mf"),
        "a8d9731e66be1021eca82d50813fdfde685c6af70df72656c0f6119b6ef7e74c",
    );
}

#[test]
fn orca_parity_auto_brim_formula_red() {
    run(
        include_bytes!("../../../../tests/parity/per-object-skirt-brim-types/auto-formula.3mf"),
        "fca874b8abdfee33e7e9c2108fea92b6bbe3dae74c3d3f7ba4458dd36cbcaaeb",
    );
}

fn run(project: &[u8], expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, "auto-brim-combined-red-");
    assert!(
        case.native.contains(";TYPE:Brim\n"),
        "Native AutoBrim active"
    );
    assert!(
        case.native.contains(";TYPE:Skirt\n"),
        "Native combined skirt active"
    );
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 17, 0, 0, 0,
    ));
}
