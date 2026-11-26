//! Continuous gyroid `AdaptiveInfillField` parity cases against pinned OrcaSlicer 2.5.0.

#[path = "adaptive_infill_field/fixtures.rs"]
mod fixtures;

use ares_core::{GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};

use crate::runner::{native_api_cli::sha, native_stream_case};

const UPSTREAM: (&str, &[&str]) = (
    "adaptive-infill-field",
    &[
        "PrintConfig.hpp:177,291-308,1528-1530",
        "PrintConfig.cpp:3727-3793,9765-9770",
        "PrintObject.cpp:1274-1304,3365",
        "Fill/AdaptiveInfillField.cpp:56-776",
        "Fill/Fill.cpp:317-401,1024-1034,1387,1437-1439,1620,1660-1662",
        "Fill/FillGyroid.cpp:25-28,345-349",
    ],
);

fn sparse_moves(gcode: &str) -> Vec<(String, String)> {
    let mut sparse = false;
    let mut moves = Vec::new();
    for line in gcode.lines() {
        if let Some(role) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            sparse = role == "Sparse infill";
            continue;
        }
        if !sparse || !line.starts_with("G1 ") {
            continue;
        }
        let mut x = None;
        let mut y = None;
        let mut extrudes = false;
        for word in line.split_ascii_whitespace().skip(1) {
            if word.starts_with(';') {
                break;
            }
            if let Some(v) = word.strip_prefix('X') {
                x = Some(v.to_owned());
            } else if let Some(v) = word.strip_prefix('Y') {
                y = Some(v.to_owned());
            } else if let Some(v) = word.strip_prefix('E') {
                extrudes = v.parse::<f64>().unwrap_or(0.0) > 0.0;
            }
        }
        if let (Some(x), Some(y), true) = (x, y, extrudes) {
            moves.push((x, y));
        }
    }
    moves
}

fn check_case(label: &str, project: &[u8], expected_lines: &[&str]) -> Vec<(String, String)> {
    let captured = std::cell::RefCell::new(Vec::new());
    native_stream_case::check(
        UPSTREAM,
        label,
        project,
        &sha(project),
        (false, |reference, _| {
            for expected in expected_lines {
                assert!(
                    reference.native.lines().any(|line| line == *expected),
                    "missing {expected:?} in Native output at {:?}",
                    reference.root
                );
            }
            *captured.borrow_mut() = sparse_moves(&reference.native);
        }),
    );
    captured.into_inner()
}

#[test]
fn adaptive_gyroid_lobes_gradients_match_native() {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    let disabled = fixtures::build_3mf(&cube, &[], &[]);
    let disabled_moves = check_case(
        "gyroid-disabled",
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
                ("adaptive_infill_mode", Some(json!("lobes"))),
                ("adaptive_infill_gradient", Some(json!(gradient))),
            ],
            &[],
        );
        let gradient_line = format!("; adaptive_infill_gradient = {gradient}");
        let moves = check_case(
            &format!("gyroid-lobes-{gradient}"),
            &project,
            &[
                "; sparse_infill_pattern = gyroid",
                "; adaptive_infill_mode = lobes",
                &gradient_line,
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!moves.is_empty());
        assert_ne!(
            moves, disabled_moves,
            "Native adaptive lobes ({gradient}) must alter sparse infill"
        );
    }
}

#[test]
fn adaptive_gyroid_distance_warp_and_smooth_blend_match_native() {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    for mode in ["distance_warp", "smooth_blend"] {
        let project =
            fixtures::build_3mf(&cube, &[("adaptive_infill_mode", Some(json!(mode)))], &[]);
        let mode_line = format!("; adaptive_infill_mode = {mode}");
        let moves = check_case(
            &format!("gyroid-{mode}"),
            &project,
            &[
                "; sparse_infill_pattern = gyroid",
                &mode_line,
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!moves.is_empty());
    }
}

#[test]
fn adaptive_gyroid_normal_axes_match_native() {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    for mode in ["normal_z", "normal_y", "normal_x"] {
        let project =
            fixtures::build_3mf(&cube, &[("adaptive_infill_mode", Some(json!(mode)))], &[]);
        let mode_line = format!("; adaptive_infill_mode = {mode}");
        let moves = check_case(
            &format!("gyroid-{mode}"),
            &project,
            &[
                "; sparse_infill_pattern = gyroid",
                &mode_line,
                ";TYPE:Sparse infill",
            ],
        );
        assert!(!moves.is_empty());
    }
}

#[test]
fn adaptive_gyroid_multi_lobe_and_multi_body_match_native() {
    let dumbbell = fixtures::necked_dumbbell_mesh();
    let dumbbell_project = fixtures::build_3mf(
        &dumbbell,
        &[("adaptive_infill_mode", Some(json!("lobes")))],
        &[],
    );
    let dumbbell_moves = check_case(
        "gyroid-dumbbell-lobes",
        &dumbbell_project,
        &["; adaptive_infill_mode = lobes", ";TYPE:Sparse infill"],
    );
    assert!(!dumbbell_moves.is_empty());

    let two_bodies = fixtures::two_bodies_mesh();
    let bodies_project = fixtures::build_3mf(
        &two_bodies,
        &[("adaptive_infill_mode", Some(json!("lobes")))],
        &[],
    );
    let bodies_moves = check_case(
        "gyroid-two-bodies-lobes",
        &bodies_project,
        &["; adaptive_infill_mode = lobes", ";TYPE:Sparse infill"],
    );
    assert!(!bodies_moves.is_empty());
}

#[test]
fn adaptive_gyroid_overrides_legacy_precedence_and_fallback_match_native() {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    let obj_override = fixtures::build_3mf(
        &cube,
        &[],
        &[
            ("adaptive_infill_mode", "lobes"),
            ("adaptive_infill_interior_density", "10%"),
        ],
    );
    check_case(
        "gyroid-object-override",
        &obj_override,
        &["; adaptive_infill_mode = disabled", ";TYPE:Sparse infill"],
    );

    let legacy_patch: &[(&str, Option<Value>)] = &[
        ("adaptive_infill_mode", None),
        ("adaptive_infill_interior_density", None),
        ("adaptive_infill_gradient", None),
        ("tpms_adaptive", Some(json!("lobes"))),
        ("tpms_interior_density", Some(json!("8%"))),
        ("tpms_adaptive_gradient", Some(json!("quadratic"))),
    ];
    let legacy = fixtures::build_3mf(&cube, legacy_patch, &[]);
    check_case(
        "gyroid-legacy-keys",
        &legacy,
        &[
            "; adaptive_infill_mode = lobes",
            "; adaptive_infill_interior_density = 8%",
            "; adaptive_infill_gradient = quadratic",
        ],
    );

    let opt_precedence = fixtures::build_3mf(
        &cube,
        &[
            ("adaptive_infill_mode", Some(json!("lobes"))),
            ("gyroid_optimized", Some(json!("1"))),
        ],
        &[],
    );
    check_case(
        "gyroid-lobes-over-optimized",
        &opt_precedence,
        &[
            "; adaptive_infill_mode = lobes",
            "; gyroid_optimized = 1",
            ";TYPE:Sparse infill",
        ],
    );

    let thin = fixtures::box_mesh(20.0, 20.0, 0.4);
    let thin_lobes = fixtures::build_3mf(
        &thin,
        &[
            ("adaptive_infill_mode", Some(json!("lobes"))),
            ("top_shell_layers", Some(json!("0"))),
            ("bottom_shell_layers", Some(json!("0"))),
        ],
        &[],
    );
    let thin_disabled = fixtures::build_3mf(
        &thin,
        &[
            ("adaptive_infill_mode", Some(json!("disabled"))),
            ("top_shell_layers", Some(json!("0"))),
            ("bottom_shell_layers", Some(json!("0"))),
        ],
        &[],
    );
    let thin_lobes_moves = check_case(
        "gyroid-thin-fallback",
        &thin_lobes,
        &["; adaptive_infill_mode = lobes", ";TYPE:Sparse infill"],
    );
    let thin_disabled_moves = check_case(
        "gyroid-thin-disabled",
        &thin_disabled,
        &["; adaptive_infill_mode = disabled", ";TYPE:Sparse infill"],
    );
    assert_eq!(thin_lobes_moves, thin_disabled_moves);

    let grid_lobes = fixtures::build_3mf(
        &cube,
        &[
            ("sparse_infill_pattern", Some(json!("grid"))),
            ("adaptive_infill_mode", Some(json!("lobes"))),
        ],
        &[],
    );
    check_case(
        "grid-lobes-passthrough",
        &grid_lobes,
        &[
            "; sparse_infill_pattern = grid",
            "; adaptive_infill_mode = lobes",
        ],
    );
}

#[test]
fn adaptive_gyroid_bridge_over_infill_anchoring_match_native() {
    let stepped = fixtures::stepped_bridge_mesh();
    let project = fixtures::build_3mf(
        &stepped,
        &[("adaptive_infill_mode", Some(json!("lobes")))],
        &[],
    );
    let moves = check_case(
        "gyroid-stepped-bridge-anchoring",
        &project,
        &[
            "; sparse_infill_pattern = gyroid",
            "; adaptive_infill_mode = lobes",
            ";TYPE:Internal Bridge",
            ";TYPE:Sparse infill",
        ],
    );
    assert!(!moves.is_empty());
}

#[tokio::test]
async fn adaptive_infill_deferred_modes_report_unsupported() {
    let cube = fixtures::box_mesh(20.0, 20.0, 20.0);
    let cases = [
        ("gyroid", "stepped_shells"),
        ("tpmsd", "lobes"),
        ("tpmsfk", "lobes"),
    ];
    for (pattern, mode) in cases {
        let project = fixtures::build_3mf(
            &cube,
            &[
                ("sparse_infill_pattern", Some(json!(pattern))),
                ("adaptive_infill_mode", Some(json!(mode))),
            ],
            &[],
        );
        let result = ares_core::slice_project(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            SlicerContext::new(1),
        )
        .await;
        assert!(
            matches!(result, Err(SliceError::UnsupportedProjectFeature(_))),
            "expected UnsupportedProjectFeature for {pattern} + {mode}, got {:?}",
            result.map(|bytes| bytes.len())
        );
    }
}
