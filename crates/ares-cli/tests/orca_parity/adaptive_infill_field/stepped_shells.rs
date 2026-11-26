//! `SteppedShells` adaptive infill parity cases for gyroid, TPMS-D, and TPMS-FK.

use serde_json::json;

use super::{check_case, fixtures};

#[test]
fn adaptive_gyroid_stepped_shells_match_native() {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    let disabled = fixtures::build_3mf(&cube, &[], &[]);
    let disabled_moves = check_case(
        "gyroid-stepped-baseline",
        &disabled,
        &[
            "; sparse_infill_pattern = gyroid",
            "; adaptive_infill_mode = disabled",
            ";TYPE:Sparse infill",
        ],
    );
    assert!(!disabled_moves.is_empty());

    for gradient in ["linear", "quadratic", "exponential"] {
        let project = fixtures::build_3mf(
            &cube,
            &[
                ("adaptive_infill_mode", Some(json!("stepped_shells"))),
                ("adaptive_infill_gradient", Some(json!(gradient))),
            ],
            &[],
        );
        let gradient_line = format!("; adaptive_infill_gradient = {gradient}");
        let moves = check_case(
            &format!("gyroid-stepped-{gradient}"),
            &project,
            &[
                "; sparse_infill_pattern = gyroid",
                "; adaptive_infill_mode = stepped_shells",
                &gradient_line,
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!moves.is_empty());
        assert_ne!(
            moves, disabled_moves,
            "Native stepped_shells ({gradient}) must alter sparse infill"
        );
    }

    // Equal surface and interior density still enters `fill_adaptive_infill_shells`
    // and shrinks the single shell by half a spacing before filling.
    let equal = fixtures::build_3mf(
        &cube,
        &[
            ("adaptive_infill_mode", Some(json!("stepped_shells"))),
            ("adaptive_infill_interior_density", Some(json!("30%"))),
        ],
        &[],
    );
    let equal_moves = check_case(
        "gyroid-stepped-equal",
        &equal,
        &[
            "; adaptive_infill_mode = stepped_shells",
            "; adaptive_infill_interior_density = 30%",
            ";TYPE:Sparse infill",
        ],
    );
    assert!(!equal_moves.is_empty());
    assert_ne!(equal_moves, disabled_moves);

    // Inverted ratio: sparser surface (5%) with denser interior (30%).
    let inverted = fixtures::build_3mf(
        &cube,
        &[
            ("adaptive_infill_mode", Some(json!("stepped_shells"))),
            ("sparse_infill_density", Some(json!("5%"))),
            ("adaptive_infill_interior_density", Some(json!("30%"))),
        ],
        &[],
    );
    let inverted_moves = check_case(
        "gyroid-stepped-inverted",
        &inverted,
        &[
            "; adaptive_infill_mode = stepped_shells",
            "; sparse_infill_density = 5%",
            "; adaptive_infill_interior_density = 30%",
            ";TYPE:Sparse infill",
        ],
    );
    assert!(!inverted_moves.is_empty());

    // `gyroid_optimized` must stay suppressed even inside `fill_shell` callbacks.
    let over_optimized = fixtures::build_3mf(
        &cube,
        &[
            ("adaptive_infill_mode", Some(json!("stepped_shells"))),
            ("gyroid_optimized", Some(json!("1"))),
        ],
        &[],
    );
    let over_opt_moves = check_case(
        "gyroid-stepped-over-optimized",
        &over_optimized,
        &[
            "; adaptive_infill_mode = stepped_shells",
            "; gyroid_optimized = 1",
            ";TYPE:Sparse infill",
        ],
    );
    assert!(!over_opt_moves.is_empty());
}

#[test]
fn adaptive_gyroid_stepped_shells_bridge_anchoring_match_native() {
    let stepped = fixtures::stepped_bridge_mesh();
    let project = fixtures::build_3mf(
        &stepped,
        &[("adaptive_infill_mode", Some(json!("stepped_shells")))],
        &[],
    );
    let moves = check_case(
        "gyroid-stepped-bridge-shells",
        &project,
        &[
            "; sparse_infill_pattern = gyroid",
            "; adaptive_infill_mode = stepped_shells",
            ";TYPE:Internal Bridge",
            ";TYPE:Sparse infill",
        ],
    );
    assert!(!moves.is_empty());
}

#[test]
fn adaptive_tpmsd_and_tpmsfk_stepped_shells_match_native() {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    let stepped = fixtures::stepped_bridge_mesh();
    let thin = fixtures::box_mesh(20.0, 20.0, 0.4);

    for pattern in ["tpmsd", "tpmsfk"] {
        let pattern_line = format!("; sparse_infill_pattern = {pattern}");
        let disabled = fixtures::build_3mf(
            &cube,
            &[("sparse_infill_pattern", Some(json!(pattern)))],
            &[],
        );
        let disabled_moves = check_case(
            &format!("{pattern}-stepped-baseline"),
            &disabled,
            &[
                &pattern_line,
                "; adaptive_infill_mode = disabled",
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!disabled_moves.is_empty());

        for gradient in ["linear", "quadratic"] {
            let project = fixtures::build_3mf(
                &cube,
                &[
                    ("sparse_infill_pattern", Some(json!(pattern))),
                    ("adaptive_infill_mode", Some(json!("stepped_shells"))),
                    ("adaptive_infill_gradient", Some(json!(gradient))),
                ],
                &[],
            );
            let gradient_line = format!("; adaptive_infill_gradient = {gradient}");
            let moves = check_case(
                &format!("{pattern}-stepped-{gradient}"),
                &project,
                &[
                    &pattern_line,
                    "; adaptive_infill_mode = stepped_shells",
                    &gradient_line,
                    ";TYPE:Sparse infill",
                ],
            );
            assert!(!moves.is_empty());
            assert_ne!(
                moves, disabled_moves,
                "Native {pattern} stepped_shells ({gradient}) must alter sparse infill"
            );
        }

        let bridge = fixtures::build_3mf(
            &stepped,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!("stepped_shells"))),
            ],
            &[],
        );
        let bridge_moves = check_case(
            &format!("{pattern}-stepped-bridge"),
            &bridge,
            &[
                &pattern_line,
                "; adaptive_infill_mode = stepped_shells",
                ";TYPE:Internal Bridge",
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!bridge_moves.is_empty());

        let thin_shells = fixtures::build_3mf(
            &thin,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!("stepped_shells"))),
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
        let thin_shells_moves = check_case(
            &format!("{pattern}-thin-stepped"),
            &thin_shells,
            &[
                &pattern_line,
                "; adaptive_infill_mode = stepped_shells",
                ";TYPE:Sparse infill",
            ],
        );
        let thin_disabled_moves = check_case(
            &format!("{pattern}-thin-disabled"),
            &thin_disabled,
            &[
                &pattern_line,
                "; adaptive_infill_mode = disabled",
                ";TYPE:Sparse infill",
            ],
        );
        assert_eq!(thin_shells_moves, thin_disabled_moves);
    }
}
