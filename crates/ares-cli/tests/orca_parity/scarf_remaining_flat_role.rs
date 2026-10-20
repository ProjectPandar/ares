//! ExtrusionLoop::role / ExtrusionLoopSloped::clip_front: remaining flats own the role.
use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn scarf_remaining_flat_role_entire_loop() {
    run(
        "positive",
        include_bytes!("../../../../tests/parity/scarf-empty-flat-role/positive.3mf"),
        "a99f4752e06f36102488649a944233cacb0cb2541ec42e03fa7ed431a8ac98ee",
    );
}

#[test]
fn scarf_remaining_flat_role_minimum_exhausts_loop() {
    run(
        "length100",
        include_bytes!("../../../../tests/parity/scarf-empty-flat-role/length100.3mf"),
        "0d9d223e7276a81043f2e8640e7199bb40a1a6a49f015d159c8842633fb5f141",
    );
}

#[test]
fn scarf_remaining_flat_role_partial_control() {
    run(
        "control",
        include_bytes!("../../../../tests/parity/scarf-empty-flat-role/control.3mf"),
        "f5180678b43d3f1895b02aa0b8c4ed096d1792c41fa0ce9217791d6ef0f7f454",
    );
}

#[test]
fn scarf_remaining_flat_role_no_inner_control() {
    run(
        "no-inner-control",
        include_bytes!("../../../../tests/parity/scarf-empty-flat-role/no-inner-control.3mf"),
        "0a8f2e4ffac21a912c663cfd3498f166e533cfe86fad7f58f0a4ea9af18156e5",
    );
}

#[test]
fn scarf_remaining_flat_role_zero_gap_control() {
    run(
        "zero-gap-control",
        include_bytes!("../../../../tests/parity/scarf-empty-flat-role/zero-gap-control.3mf"),
        "5bb0a0e28d6b3ea5102b833406b9c508c25a168e959984465c6f4c2ab191c59c",
    );
}

fn run(label: &str, project: &[u8], expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, &format!("scarf-role-{label}-"));
    assert!(case.native.contains("; seam_position = aligned\n"));
    assert!(case.native.contains("; staggered_inner_seams = 0\n"));
    assert!(case.native.contains("; wipe = 0\n"));
    if label == "positive" {
        assert!(case.native.contains("; seam_slope_entire_loop = 1\n"));
        assert!(
            case.native
                .lines()
                .any(|line| line == "G1 X119.325 Y100.755 Z.8")
        );
    }
    if label == "length100" {
        assert!(case.native.contains("; seam_slope_min_length = 100\n"));
        assert!(case.native.contains("; seam_slope_entire_loop = 0\n"));
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}
