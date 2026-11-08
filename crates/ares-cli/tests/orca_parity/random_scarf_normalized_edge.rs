//! SeamPlacer.cpp:1441-1443: position + edge.normalized() * picked_len.
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
    assert_eq!(
        has_inner_wall_scarf(&case.native),
        inner,
        "Native inner-wall scarf activity"
    );
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}

fn has_inner_wall_scarf(text: &str) -> bool {
    let mut role = "";
    let mut previous_z = None;
    for line in text.lines() {
        if line == ";LAYER_CHANGE" {
            role = "";
            previous_z = None;
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        let tokens: Vec<_> = line.split(';').next().unwrap().split_whitespace().collect();
        let Some(command) = tokens.first() else {
            continue;
        };
        if !matches!(*command, "G0" | "G1" | "G2" | "G3") {
            continue;
        }
        let z = tokens
            .iter()
            .find_map(|token| token.strip_prefix('Z'))
            .map(|value| value.parse::<f64>().unwrap());
        let changes_z = previous_z
            .zip(z)
            .is_some_and(|(before, after)| before != after);
        previous_z = z.or(previous_z);
        let has_xy = tokens.iter().any(|token| token.starts_with(['X', 'Y']));
        let extrudes = tokens
            .iter()
            .find_map(|token| token.strip_prefix('E'))
            .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0);
        if *command == "G1" && role == "Inner wall" && has_xy && extrudes && changes_z {
            return true;
        }
    }
    false
}
