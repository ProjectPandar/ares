//! PrintObjectSlice.cpp:628-675 reGroupingLayerPolygons: zero-difference matches.

use crate::{
    OrcaRunner,
    auto_brim_object_speed::brim_moves,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn auto_brim_exact_group_assignment_separated() {
    run(
        "brim-exact-group-separated-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-exact-group-assignment/separated-auto.3mf"
        ),
        "8ce90e912e8fb20f3c190a2b4b7971d8ba49b1b6cd8f9f1fdc9477e5e5541096",
        true,
    );
}

#[test]
fn auto_brim_exact_group_assignment_single_control() {
    run(
        "brim-exact-group-single-",
        include_bytes!("../../../../tests/parity/auto-brim-exact-group-assignment/single-auto.3mf"),
        "e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5",
        true,
    );
}

#[test]
fn auto_brim_exact_group_assignment_overlap_control() {
    run(
        "brim-exact-group-overlap-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-exact-group-assignment/overlap-auto.3mf"
        ),
        "4b91d9a7884320b86ed44a0c112811aa90a5feb9e7f6381e40c3420b930d4a78",
        true,
    );
}

#[test]
fn auto_brim_exact_group_assignment_no_brim_control() {
    run(
        "brim-exact-group-none-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-exact-group-assignment/separated-none.3mf"
        ),
        "5b9ac2700a7365dea7b0a87aa46f6b9aeef447511de8ad1fea26a8a467bb7b02",
        false,
    );
}

fn run(prefix: &str, project: &[u8], expected_sha: &str, has_brim: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, prefix);
    let native_moves = brim_moves(&case.native);
    assert_eq!(case.native.contains(";TYPE:Brim\n"), has_brim);
    assert_eq!(!native_moves.is_empty(), has_brim);
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
    for bytes in [&api, &cli] {
        let text = std::str::from_utf8(bytes).unwrap();
        assert_eq!(text.contains(";TYPE:Brim\n"), has_brim);
        assert_eq!(brim_moves(text), native_moves);
    }
}
