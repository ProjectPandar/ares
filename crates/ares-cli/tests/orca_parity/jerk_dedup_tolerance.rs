//! `GCodeWriter.cpp:255-260` uses default `is_approx` before caching jerk.

#[test]
fn within_native_tolerance_suppresses_redundant_jerk() {
    check(
        "within-native-tolerance",
        include_bytes!("../../../../tests/parity/jerk-dedup-tolerance/within-native-tolerance.3mf"),
        "e7a5ca34d4981a559d82d6895f59f5a642ddbdb69e49aa715aef5bd5eb351a80",
        Some(false),
    );
}

#[test]
fn stock_jerk_control_stays_byte_equal() {
    check(
        "stock-control",
        include_bytes!("../../../../tests/parity/jerk-dedup-tolerance/stock-control.3mf"),
        "a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae",
        None,
    );
}

#[test]
fn equal_jerk_control_stays_byte_equal() {
    check(
        "equal-control",
        include_bytes!("../../../../tests/parity/jerk-dedup-tolerance/equal-control.3mf"),
        "afe37155ecbdb81ebe72e49a8e5dd5709b359f0e63f73d0e50a3689f12d2fe2b",
        Some(false),
    );
}

#[test]
fn above_native_tolerance_emits_changed_jerk() {
    check(
        "above-native-tolerance",
        include_bytes!("../../../../tests/parity/jerk-dedup-tolerance/above-native-tolerance.3mf"),
        "64abf2e38a99dfe87d84d3380149794bfd37779bf6d3d8821445e22bb0c66847",
        Some(true),
    );
}

#[test]
fn within_both_tolerances_control_stays_byte_equal() {
    check(
        "within-both-tolerances",
        include_bytes!("../../../../tests/parity/jerk-dedup-tolerance/within-both-tolerances.3mf"),
        "335b65c5f33a7ca0a71f72b926c325965179ef3e6eca912a7bbb341f382f33ff",
        Some(false),
    );
}

fn check(label: &str, project: &[u8], sha: &str, distinct_jerk: Option<bool>) {
    crate::runner::native_stream_case::check(
        (
            "jerk-dedup-tolerance",
            &[
                "GCodeWriter.cpp:255-260::set_jerk_xy",
                "libslic3r.h:306-309::is_approx",
                "libslic3r.h:52::EPSILON",
            ],
        ),
        label,
        project,
        sha,
        (false, |reference, _| {
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line.starts_with("M205 X"))
            );
            if let Some(distinct) = distinct_jerk {
                let body = reference.native.split_once(";TYPE:Custom\n").unwrap().1;
                let commands: Vec<_> = body
                    .lines()
                    .filter(|line| line.starts_with("M205 X"))
                    .collect();
                assert!(
                    !commands.is_empty(),
                    "Native must configure controlled body jerk"
                );
                assert_eq!(
                    commands.windows(2).any(|pair| pair[0] != pair[1]),
                    distinct,
                    "Native changed-jerk admission must match the source tolerance"
                );
            }
        }),
    );
}
