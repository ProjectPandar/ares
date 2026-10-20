//! Shared EFC outline witness assertions around Native/API/CLI strict parity.

use crate::{
    OrcaRunner,
    auto_brim_object_speed::brim_moves,
    runner::native_api_cli::{NativeCase, sha},
};

pub(super) fn run(
    prefix: &str,
    project: &[u8],
    expected_sha: &str,
    raw_control: Option<(&[u8], &str)>,
    extra_assertions: impl Fn(&str),
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, prefix);
    let native_moves = brim_moves(&case.native);
    assert!(case.native.contains(";TYPE:Brim\n"));
    assert!(!native_moves.is_empty());
    extra_assertions(&case.native);
    if let Some((raw_project, raw_prefix)) = raw_control {
        let raw = NativeCase::slice(&runner, raw_project, raw_prefix);
        let raw_moves = brim_moves(&raw.native);
        assert!(!raw_moves.is_empty());
        assert_ne!(
            native_moves, raw_moves,
            "Native EFC outline changes actual Brim extrusion"
        );
    }
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
    for bytes in [&api, &cli] {
        let text = std::str::from_utf8(bytes).unwrap();
        assert!(text.contains(";TYPE:Brim\n"));
        assert_eq!(brim_moves(text), native_moves);
        extra_assertions(text);
    }
}
