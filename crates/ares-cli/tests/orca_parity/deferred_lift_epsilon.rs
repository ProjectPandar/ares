//! `GCodeWriter.cpp:685-710`: deferred lift consumption is strictly above EPSILON.

#[test]
fn below_epsilon_pending_lift_is_not_consumed() {
    check(
        "below-epsilon",
        include_bytes!("../../../../tests/parity/deferred-lift-epsilon/below-epsilon.3mf"),
        "7614d97c841ae5300be6f4aa219e05241ef6f876c199a084a38afc323fc682c5",
        false,
    );
}

#[test]
fn at_epsilon_pending_lift_is_not_consumed() {
    check(
        "at-epsilon",
        include_bytes!("../../../../tests/parity/deferred-lift-epsilon/at-epsilon.3mf"),
        "efec866bdcb60c0e523c064fa73bb298eb11298c469dad7610e1e6d01e1892ba",
        false,
    );
}

#[test]
fn zero_hop_control_stays_byte_equal() {
    check(
        "zero-control",
        include_bytes!("../../../../tests/parity/deferred-lift-epsilon/zero-control.3mf"),
        "6ec170cd2be5d9598f978c3363587e571814ab118b704a81b636c53c71244f8d",
        false,
    );
}

#[test]
fn above_epsilon_pending_lift_is_consumed() {
    check(
        "above-epsilon",
        include_bytes!("../../../../tests/parity/deferred-lift-epsilon/above-epsilon.3mf"),
        "a5320b5b36a1596031dc0d6bb25c74447ccfbb948169ebfdf1f0b6192aa9c2fa",
        true,
    );
}

#[test]
fn default_hop_control_stays_byte_equal() {
    check(
        "stock-control",
        include_bytes!("../../../../tests/parity/deferred-lift-epsilon/stock-control.3mf"),
        "a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae",
        true,
    );
}

fn check(label: &str, project: &[u8], sha: &str, consumes_lift: bool) {
    crate::runner::native_stream_case::check(
        (
            "deferred-lift-epsilon",
            &[
                "libslic3r.h:52::EPSILON",
                "GCodeWriter.cpp:625-648::lazy_lift",
                "GCodeWriter.cpp:685-710::travel_to_xyz",
                "GCodeWriter.cpp:730-808::travel_to_xyz",
            ],
        ),
        label,
        project,
        sha,
        (false, |reference, _| {
            let after_marker = reference
                .native
                .split_once(";_SET_FAN_SPEED_CHANGING_LAYER\n")
                .unwrap()
                .1;
            let first_move = after_marker.lines().next().unwrap();
            assert!(first_move.contains(" F"), "Native approach sets feedrate");
            if consumes_lift {
                assert!(
                    first_move.starts_with("G1 Z"),
                    "Native consumes normal lift"
                );
                assert!(!first_move.contains(" X") && !first_move.contains(" Y"));
            } else {
                assert!(first_move.starts_with("G1 X"), "Native approaches in XY");
                assert!(first_move.contains(" Y") && !first_move.contains(" Z"));
            }
        }),
    );
}
