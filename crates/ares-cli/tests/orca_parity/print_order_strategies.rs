//! Whole-G-code parity cases for `print_order = snake` and `print_order = best_of`
//! against pinned OrcaSlicer 2.5.0.

#[path = "print_order_strategies/fixtures.rs"]
mod fixtures;

use fixtures::{BoxSpec, ObjectSpec};
use serde_json::json;

use crate::runner::{native_api_cli::sha, native_stream_case};

const UPSTREAM: (&str, &[&str]) = (
    "print-order-strategies",
    &[
        "GCode/OrderingStrategies.hpp:26-148",
        "GCode/OrderingStrategies.cpp:30-104,115-228,282-550",
        "GCode.cpp:3614-3616,3680-3689,7270-7392",
    ],
);

const SUBGRID_18: &[[f64; 2]] = &[
    [92.0, 62.0],
    [164.0, 146.0],
    [20.0, 104.0],
    [56.0, 20.0],
    [20.0, 62.0],
    [128.0, 62.0],
    [20.0, 146.0],
    [164.0, 20.0],
    [56.0, 146.0],
    [92.0, 104.0],
    [56.0, 104.0],
    [164.0, 104.0],
    [20.0, 188.0],
    [56.0, 62.0],
    [92.0, 20.0],
    [56.0, 188.0],
    [92.0, 146.0],
    [92.0, 188.0],
];

const CLUSTERED_14: &[[f64; 2]] = &[
    [30.0, 30.0],
    [42.0, 35.0],
    [35.0, 48.0],
    [180.0, 32.0],
    [192.0, 40.0],
    [175.0, 52.0],
    [100.0, 95.0],
    [112.0, 108.0],
    [95.0, 118.0],
    [32.0, 175.0],
    [46.0, 185.0],
    [185.0, 170.0],
    [195.0, 188.0],
    [170.0, 192.0],
];

fn parse_extruding_xy(line: &str) -> Option<(i64, i64)> {
    let (mut x, mut y, mut extrudes) = (None, None, false);
    for word in line.split_ascii_whitespace().skip(1) {
        if word.starts_with(';') {
            break;
        }
        if let Some(v) = word.strip_prefix('X') {
            x = v.parse::<f64>().ok();
        } else if let Some(v) = word.strip_prefix('Y') {
            y = v.parse::<f64>().ok();
        } else if let Some(v) = word.strip_prefix('E') {
            extrudes = v.parse::<f64>().unwrap_or(0.0) > 0.0;
        }
    }
    extrudes.then(|| Some((x?.round() as i64, y?.round() as i64)))?
}

fn wall_starts_by_layer(gcode: &str) -> Vec<Vec<(i64, i64)>> {
    let mut layers = Vec::new();
    let mut current = Vec::new();
    let mut expect_xy = false;
    for line in gcode.lines() {
        if line.starts_with(";LAYER_CHANGE") {
            if !current.is_empty() {
                layers.push(std::mem::take(&mut current));
            }
        } else if line == ";TYPE:Outer wall" {
            expect_xy = true;
        } else if expect_xy
            && line.starts_with("G1 ")
            && let Some(xy) = parse_extruding_xy(line)
        {
            current.push(xy);
            expect_xy = false;
        }
    }
    if !current.is_empty() {
        layers.push(current);
    }
    layers
}

fn check_case(
    label: &str,
    project: &[u8],
    print_order: &str,
    assert_two_tools: bool,
) -> Vec<Vec<(i64, i64)>> {
    let captured = std::cell::RefCell::new(Vec::new());
    let expected_line = format!("; print_order = {print_order}");
    native_stream_case::check(
        UPSTREAM,
        label,
        project,
        &sha(project),
        (assert_two_tools, |reference, _| {
            assert!(
                reference.native.lines().any(|line| line == expected_line),
                "missing {expected_line:?} in Native output at {:?}",
                reference.root
            );
            *captured.borrow_mut() = wall_starts_by_layer(&reference.native);
        }),
    );
    captured.into_inner()
}

fn instances_z(points: &[[f64; 2]]) -> Vec<[f64; 3]> {
    points.iter().map(|&[x, y]| [x, y, 0.3]).collect()
}

#[test]
fn print_order_subgrid_snake_wins_best_of() {
    let cube = [BoxSpec::cube4(0.0, 0.0)];
    let insts = instances_z(SUBGRID_18);

    let default_proj =
        fixtures::build_single_object_3mf(&cube, &insts, &[("print_order", json!("default"))]);
    let snake_proj =
        fixtures::build_single_object_3mf(&cube, &insts, &[("print_order", json!("snake"))]);
    let best_of_proj =
        fixtures::build_single_object_3mf(&cube, &insts, &[("print_order", json!("best_of"))]);

    let default_layers = check_case("subgrid18-default", &default_proj, "default", false);
    let snake_layers = check_case("subgrid18-snake", &snake_proj, "snake", false);
    let best_of_layers = check_case("subgrid18-best-of", &best_of_proj, "best_of", false);

    assert_eq!(default_layers[0].len(), 18);
    assert_ne!(snake_layers, default_layers);
    assert_eq!(best_of_layers, snake_layers);
    assert_ne!(best_of_layers, default_layers);
}

#[test]
fn print_order_clustered_and_small_grid_default_wins_best_of() {
    let cube = [BoxSpec::cube4(0.0, 0.0)];

    // Irregular 14-point layout (> 10): hits fallback row threshold and default beats snake.
    let clustered = instances_z(CLUSTERED_14);
    let cl_def = check_case(
        "clustered14-default",
        &fixtures::build_single_object_3mf(&cube, &clustered, &[("print_order", json!("default"))]),
        "default",
        false,
    );
    let cl_snake = check_case(
        "clustered14-snake",
        &fixtures::build_single_object_3mf(&cube, &clustered, &[("print_order", json!("snake"))]),
        "snake",
        false,
    );
    let cl_best = check_case(
        "clustered14-best-of",
        &fixtures::build_single_object_3mf(&cube, &clustered, &[("print_order", json!("best_of"))]),
        "best_of",
        false,
    );
    assert_eq!(cl_def[0].len(), 14);
    assert_ne!(cl_snake, cl_def);
    assert_eq!(cl_best, cl_def);

    // 2x3 grid (<= 10): default uses tsp_exact_cycle + nearer-neighbor reversal; snake rotates only.
    let small_grid = instances_z(&[
        [40.0, 50.0],
        [100.0, 50.0],
        [160.0, 50.0],
        [40.0, 110.0],
        [100.0, 110.0],
        [160.0, 110.0],
    ]);
    let sm_def = check_case(
        "small6-default",
        &fixtures::build_single_object_3mf(
            &cube,
            &small_grid,
            &[("print_order", json!("default"))],
        ),
        "default",
        false,
    );
    let sm_snake = check_case(
        "small6-snake",
        &fixtures::build_single_object_3mf(&cube, &small_grid, &[("print_order", json!("snake"))]),
        "snake",
        false,
    );
    let sm_best = check_case(
        "small6-best-of",
        &fixtures::build_single_object_3mf(
            &cube,
            &small_grid,
            &[("print_order", json!("best_of"))],
        ),
        "best_of",
        false,
    );
    assert_eq!(sm_def[0].len(), 6);
    assert_ne!(sm_snake, sm_def);
    assert_eq!(sm_best, sm_def);
}

#[test]
fn print_order_small_count_and_degenerate_rows() {
    let cube = [BoxSpec::cube4(0.0, 0.0)];

    // n = 3 exercises early returns in tsp_2opt_improve (<= 2) and tsp_remove_crossings (<= 3).
    let tiny = instances_z(&[[40.0, 40.0], [120.0, 40.0], [80.0, 120.0]]);
    for po in ["snake", "best_of"] {
        let proj = fixtures::build_single_object_3mf(&cube, &tiny, &[("print_order", json!(po))]);
        let layers = check_case(&format!("tiny3-{po}"), &proj, po, false);
        assert_eq!(layers[0].len(), 3);
    }

    // Single horizontal row (unique_ys.len() == 1) and uniform diagonal (max_gap < min_gap * 2).
    let single_row = (0..12)
        .map(|i| [20.0 + 15.0 * i as f64, 100.0, 0.3])
        .collect::<Vec<_>>();
    let diagonal = (0..12)
        .map(|i| [25.0 + 14.0 * i as f64, 25.0 + 14.0 * i as f64, 0.3])
        .collect::<Vec<_>>();
    for (tag, insts) in [("single-row12", &single_row), ("diag12", &diagonal)] {
        for po in ["snake", "best_of"] {
            let proj =
                fixtures::build_single_object_3mf(&cube, insts, &[("print_order", json!(po))]);
            let layers = check_case(&format!("{tag}-{po}"), &proj, po, false);
            assert_eq!(layers[0].len(), 12);
        }
    }
}

#[test]
fn print_order_stepped_multi_island_and_interleaved_objects() {
    // Stepped single-object multi-island mesh: bottom layers have all 18 SUBGRID_18 islands
    // (best_of picks snake); top layers have 6 islands (2x3 grid).
    let cx0 = 92.0;
    let cy0 = 104.0;
    let tall = [
        [20.0, 104.0],
        [92.0, 104.0],
        [164.0, 104.0],
        [20.0, 146.0],
        [92.0, 146.0],
        [164.0, 146.0],
    ];
    let stepped = SUBGRID_18
        .iter()
        .map(|&[x, y]| BoxSpec {
            cx: x - cx0,
            cy: y - cy0,
            z0: -0.4,
            z1: if tall.contains(&[x, y]) { 0.4 } else { 0.0 },
            sx: 4.0,
            sy: 4.0,
        })
        .collect::<Vec<_>>();
    let st_def = check_case(
        "stepped-default",
        &fixtures::build_single_object_3mf(
            &stepped,
            &[[cx0, cy0, 0.4]],
            &[("print_order", json!("default"))],
        ),
        "default",
        false,
    );
    let st_snake = check_case(
        "stepped-snake",
        &fixtures::build_single_object_3mf(
            &stepped,
            &[[cx0, cy0, 0.4]],
            &[("print_order", json!("snake"))],
        ),
        "snake",
        false,
    );
    let st_best = check_case(
        "stepped-best-of",
        &fixtures::build_single_object_3mf(
            &stepped,
            &[[cx0, cy0, 0.4]],
            &[("print_order", json!("best_of"))],
        ),
        "best_of",
        false,
    );
    assert_eq!(st_best[0].len(), 18);
    assert_eq!(st_best[0], st_snake[0]);
    assert_ne!(st_best[0], st_def[0]);
    assert_eq!(st_best.last().unwrap().len(), 6);

    // Two distinct objects whose 18 islands interleave on one filament.
    let pts_a = SUBGRID_18.iter().step_by(2).copied().collect::<Vec<_>>();
    let pts_b = SUBGRID_18
        .iter()
        .skip(1)
        .step_by(2)
        .copied()
        .collect::<Vec<_>>();
    let center = |pts: &[[f64; 2]]| {
        let min_x = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let max_x = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let min_y = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
        let max_y = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0]
    };
    let [cxa, cya] = center(&pts_a);
    let [cxb, cyb] = center(&pts_b);
    let boxes_a = pts_a
        .iter()
        .map(|&[x, y]| BoxSpec::cube4(x - cxa, y - cya))
        .collect::<Vec<_>>();
    let boxes_b = pts_b
        .iter()
        .map(|&[x, y]| BoxSpec::cube4(x - cxb, y - cyb))
        .collect::<Vec<_>>();
    for po in ["snake", "best_of"] {
        let proj = fixtures::build_two_object_3mf(
            ObjectSpec {
                boxes: &boxes_a,
                instances: &[[cxa, cya, 0.3]],
                extruder: 1,
            },
            ObjectSpec {
                boxes: &boxes_b,
                instances: &[[cxb, cyb, 0.3]],
                extruder: 1,
            },
            &[("print_order", json!(po))],
        );
        let layers = check_case(&format!("interleaved-2obj-{po}"), &proj, po, false);
        assert_eq!(layers[0].len(), 18);
    }
}

#[test]
fn print_order_two_filaments_and_by_object() {
    let cube = [BoxSpec::cube4(0.0, 0.0)];

    // Multi-filament: T0 owns 18 instances (snake beats default), T1 owns 6 instances.
    let t0_insts = instances_z(SUBGRID_18);
    let t1_insts = instances_z(&[
        [128.0, 20.0],
        [128.0, 104.0],
        [128.0, 146.0],
        [128.0, 188.0],
        [164.0, 62.0],
        [164.0, 188.0],
    ]);
    for po in ["snake", "best_of"] {
        let proj = fixtures::build_two_object_3mf(
            ObjectSpec {
                boxes: &cube,
                instances: &t0_insts,
                extruder: 1,
            },
            ObjectSpec {
                boxes: &cube,
                instances: &t1_insts,
                extruder: 2,
            },
            &[("print_order", json!(po))],
        );
        let layers = check_case(&format!("two-filaments-{po}"), &proj, po, true);
        assert_eq!(layers[0].len(), 24);
    }

    // Sequential by-object accepts snake and best_of while keeping upstream model order.
    let seq_insts = instances_z(&[[40.0, 40.0], [110.0, 40.0], [180.0, 180.0]]);
    for po in ["snake", "best_of"] {
        let proj = fixtures::build_single_object_3mf(
            &cube,
            &seq_insts,
            &[
                ("print_order", json!(po)),
                ("print_sequence", json!("by object")),
            ],
        );
        let layers = check_case(&format!("by-object-{po}"), &proj, po, false);
        assert_eq!(layers.len(), 9);
    }
}
