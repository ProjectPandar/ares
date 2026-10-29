//! SeamPlacer.cpp:1604-1619: unscale/f32/interpolate/rescale staggering.
use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn staggered_seam_interpolation_matches_native() {
    run(
        include_bytes!("../../../../tests/parity/random-seam-precision/random-staggered.3mf"),
        "51f3d3f56dddf9ebe6c2bf70e863ab68f8c0817519baa421c6bc42f702f5c3a7",
        true,
    );
}

#[test]
fn staggered_seam_interpolation_unstaggered_control() {
    run(
        include_bytes!("../../../../tests/parity/random-seam-precision/random-control.3mf"),
        "d0ae70c9b0869fc806f292b25aaa5bf3837a0148b2972c4a6c2449fb79de8a85",
        false,
    );
}

fn run(project: &[u8], expected_sha: &str, staggered: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(
        &runner,
        project,
        if staggered {
            "stagger-positive-"
        } else {
            "stagger-control-"
        },
    );
    assert!(case.native.contains(if staggered {
        "; staggered_inner_seams = 1\n"
    } else {
        "; staggered_inner_seams = 0\n"
    }));
    if staggered {
        assert!(
            case.native
                .lines()
                .any(|line| line == "G1 X100.675 Y118.143 E.0392")
        );
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}
