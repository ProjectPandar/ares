//! Continuous adaptive infill parity cases for TPMS-D and TPMS-FK.

use serde_json::json;

use super::{check_case, fixtures};

fn check_continuous_pattern(pattern: &str) {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    let pattern_line = format!("; sparse_infill_pattern = {pattern}");
    let disabled = fixtures::build_3mf(
        &cube,
        &[("sparse_infill_pattern", Some(json!(pattern)))],
        &[],
    );
    let disabled_moves = check_case(
        &format!("{pattern}-disabled"),
        &disabled,
        &[
            &pattern_line,
            "; adaptive_infill_mode = disabled",
            ";TYPE:Sparse infill",
        ],
    );
    assert!(!disabled_moves.is_empty());

    for gradient in ["linear", "quadratic", "exponential"] {
        let project = fixtures::build_3mf(
            &cube,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!("lobes"))),
                ("adaptive_infill_gradient", Some(json!(gradient))),
            ],
            &[],
        );
        let gradient_line = format!("; adaptive_infill_gradient = {gradient}");
        let moves = check_case(
            &format!("{pattern}-lobes-{gradient}"),
            &project,
            &[
                &pattern_line,
                "; adaptive_infill_mode = lobes",
                &gradient_line,
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!moves.is_empty());
        assert_ne!(
            moves, disabled_moves,
            "Native {pattern} lobes ({gradient}) must alter sparse infill"
        );
    }

    for mode in [
        "distance_warp",
        "smooth_blend",
        "normal_z",
        "normal_y",
        "normal_x",
    ] {
        let project = fixtures::build_3mf(
            &cube,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!(mode))),
            ],
            &[],
        );
        let mode_line = format!("; adaptive_infill_mode = {mode}");
        let moves = check_case(
            &format!("{pattern}-{mode}"),
            &project,
            &[&pattern_line, &mode_line, ";TYPE:Sparse infill"],
        );
        assert!(!moves.is_empty());
        assert_ne!(
            moves, disabled_moves,
            "Native {pattern} ({mode}) must alter sparse infill"
        );
    }
}

#[test]
fn adaptive_tpmsd_continuous_modes_match_native() {
    check_continuous_pattern("tpmsd");
}

#[test]
fn adaptive_tpmsfk_continuous_modes_match_native() {
    check_continuous_pattern("tpmsfk");
}

#[test]
fn adaptive_tpmsd_and_tpmsfk_bridge_anchoring_and_thin_fallback_match_native() {
    let stepped = fixtures::stepped_bridge_mesh();
    let thin = fixtures::box_mesh(20.0, 20.0, 0.4);

    for pattern in ["tpmsd", "tpmsfk"] {
        let pattern_line = format!("; sparse_infill_pattern = {pattern}");
        let bridge = fixtures::build_3mf(
            &stepped,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!("lobes"))),
            ],
            &[],
        );
        let bridge_moves = check_case(
            &format!("{pattern}-lobes-bridge"),
            &bridge,
            &[
                &pattern_line,
                "; adaptive_infill_mode = lobes",
                ";TYPE:Internal Bridge",
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!bridge_moves.is_empty());

        let thin_lobes = fixtures::build_3mf(
            &thin,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!("lobes"))),
                ("top_shell_layers", Some(json!("0"))),
                ("bottom_shell_layers", Some(json!("0"))),
            ],
            &[],
        );
        let thin_disabled = fixtures::build_3mf(
            &thin,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!("disabled"))),
                ("top_shell_layers", Some(json!("0"))),
                ("bottom_shell_layers", Some(json!("0"))),
            ],
            &[],
        );
        let thin_lobes_moves = check_case(
            &format!("{pattern}-thin-lobes"),
            &thin_lobes,
            &[
                &pattern_line,
                "; adaptive_infill_mode = lobes",
                ";TYPE:Sparse infill",
            ],
        );
        let thin_disabled_moves = check_case(
            &format!("{pattern}-thin-lobes-disabled"),
            &thin_disabled,
            &[
                &pattern_line,
                "; adaptive_infill_mode = disabled",
                ";TYPE:Sparse infill",
            ],
        );
        assert_eq!(thin_lobes_moves, thin_disabled_moves);
    }
}
