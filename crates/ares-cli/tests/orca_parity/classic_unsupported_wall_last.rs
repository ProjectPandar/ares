//! Beta `PerimeterGenerator.cpp:1391-1455,1965` and GCode deferred passes.

use crate::runner::native_stream_case;

enum NativeOrder {
    InfillBeforeWalls,
    WallsBeforeInfill,
    AnchoredBeforeOverhang,
}

fn check(case: (&str, &[u8], &str), order: NativeOrder) {
    let (label, input, hash) = case;
    native_stream_case::check(
        (
            "classic-unsupported-wall-last",
            &[
                "PerimeterGenerator.cpp::defer_unsupported_loops:1391-1455;Classic call:1965",
                "ExtrusionEntity.hpp::ExtrusionLoop::print_after_infill:474",
                "GCode.cpp::deferred perimeter passes:7742-7755,8039-8053,8889-8933",
            ],
        ),
        label,
        input,
        hash,
        (false, |reference, _| {
            assert!(reference.native.contains("; wall_generator = classic\n"));
            let enabled = !matches!(order, NativeOrder::WallsBeforeInfill);
            assert!(reference.native.contains(if enabled {
                "; unsupported_wall_last = 1\n"
            } else {
                "; unsupported_wall_last = 0\n"
            }));
            let layer = reference.native.split(";Z:1.4\n").nth(1).unwrap();
            let layer = layer.split(";LAYER_CHANGE\n").next().unwrap();
            let roles: Vec<_> = layer
                .lines()
                .filter_map(|line| line.strip_prefix(";TYPE:"))
                .collect();
            let infill = roles
                .iter()
                .position(|role| {
                    matches!(
                        *role,
                        "Bridge"
                            | "Internal Bridge"
                            | "Internal solid infill"
                            | "Bottom surface"
                            | "Sparse infill"
                    )
                })
                .unwrap();
            let overhang = roles
                .iter()
                .position(|role| *role == "Overhang wall")
                .unwrap();
            match order {
                NativeOrder::InfillBeforeWalls => assert!(infill < overhang),
                NativeOrder::WallsBeforeInfill => assert!(overhang < infill),
                NativeOrder::AnchoredBeforeOverhang => {
                    assert_eq!(roles[0], "Inner wall");
                    assert!(overhang < infill);
                }
            }
        }),
    );
}

#[test]
fn unattached_loop_waits_for_infill_complete_output() {
    check(
        (
            "unattached-ledge",
            include_bytes!(
                "../../../../tests/parity/classic-unsupported-wall-last/unattached-ledge.3mf"
            ),
            "2520f0f1d96e28c610c1ed40435ca268da45a8eea5c401cd727c6b1262e0a611",
        ),
        NativeOrder::InfillBeforeWalls,
    );
}

#[test]
fn disabled_option_preserves_complete_output() {
    check(
        (
            "disabled-control",
            include_bytes!(
                "../../../../tests/parity/classic-unsupported-wall-last/disabled-control.3mf"
            ),
            "be3d9503721e1a2236064a7d80dacb6f90907b8a01acdbf5898ac159b84ef833",
        ),
        NativeOrder::WallsBeforeInfill,
    );
}

#[test]
fn attached_wall_stack_precedes_infill_complete_output() {
    check(
        (
            "attached-stack",
            include_bytes!(
                "../../../../tests/parity/classic-unsupported-wall-last/attached-stack.3mf"
            ),
            "2aad0f555101c289b8cd1698bddda7bb520c3bbac58f51b11e30311dc411149c",
        ),
        NativeOrder::AnchoredBeforeOverhang,
    );
}
