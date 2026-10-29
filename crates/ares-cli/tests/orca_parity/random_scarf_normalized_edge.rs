//! SeamPlacer.cpp:1004-1005: position + edge.normalized() * picked_len.
use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn random_scarf_normalized_edge_matches_native() {
    run(
        include_bytes!("../../../../tests/parity/random-seam-inner-scarf/positive.3mf"),
        "51bc0bfb1aa3a007d8990b33aecd8ee2b173fc82221655ad3918da51b86851a1",
        true,
    );
}

#[test]
fn random_scarf_normalized_edge_no_inner_control() {
    run(
        include_bytes!("../../../../tests/parity/random-seam-inner-scarf/control.3mf"),
        "2181c5e766ec4d72ccae8a7ec2fbd127c7dde0f7447b9b340d840f9ea1b7f580",
        false,
    );
}

fn run(project: &[u8], expected_sha: &str, inner: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(
        &runner,
        project,
        if inner {
            "random-scarf-positive-"
        } else {
            "random-scarf-control-"
        },
    );
    assert!(case.native.contains(if inner {
        "; seam_slope_inner_walls = 1\n"
    } else {
        "; seam_slope_inner_walls = 0\n"
    }));
    assert!(case.native.contains("; wipe = 0\n"));
    if inner {
        assert!(
            case.native
                .lines()
                .any(|line| line == "G1 X114.679 Y118.918 Z18.329 E.02273")
        );
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}
