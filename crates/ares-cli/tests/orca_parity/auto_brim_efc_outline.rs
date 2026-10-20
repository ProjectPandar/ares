//! Brim.cpp:504-510: AutoBrim consumes EFC-clipped per-group polygons.

use crate::{
    OrcaRunner,
    auto_brim_object_speed::brim_moves,
    runner::native_api_cli::{NativeCase, sha},
};

const RAW_EFC: &[u8] =
    include_bytes!("../../../../tests/parity/auto-brim-efc-outline/auto-efc0p3-outline0.3mf");

#[test]
fn auto_brim_efc_outline_active() {
    run(
        "auto-brim-efc-active-",
        include_bytes!("../../../../tests/parity/auto-brim-efc-outline/auto-efc0p3-outline1.3mf"),
        "bfe65cf39ea2c485c016aa6563e788679826ff17ae4fdfa68cd69b3d17617b93",
        true,
    );
}

#[test]
fn auto_brim_efc_outline_zero_disabled_control() {
    run(
        "auto-brim-efc-zero-off-",
        include_bytes!("../../../../tests/parity/auto-brim-efc-outline/auto-efc0-outline0.3mf"),
        "1faebd4419d3310ad0d43c73a10bc7d85cda391b5c8b2faa128ec7e250833fd0",
        false,
    );
}

#[test]
fn auto_brim_efc_outline_zero_enabled_control() {
    run(
        "auto-brim-efc-zero-on-",
        include_bytes!("../../../../tests/parity/auto-brim-efc-outline/auto-efc0-outline1.3mf"),
        "895dfab6a6438a91d475b1fb2fd8cecfbc64ac5dc672dad9b0bd88d12b2f07a0",
        false,
    );
}

#[test]
fn auto_brim_efc_outline_flag_disabled_control() {
    run(
        "auto-brim-efc-flag-off-",
        RAW_EFC,
        "0e4066dedce3940dde14653f8c081b6c0cdd771b6f3b8e5f2ee851e12691ed61",
        false,
    );
}

fn run(prefix: &str, project: &[u8], expected_sha: &str, changed_outline: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, prefix);
    let native_moves = brim_moves(&case.native);
    assert!(case.native.contains(";TYPE:Brim\n"));
    assert!(!native_moves.is_empty());
    if changed_outline {
        let raw = NativeCase::slice(&runner, RAW_EFC, "auto-brim-efc-native-raw-");
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
    }
}
