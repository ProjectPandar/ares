//! `GCodeWriter.cpp:333-338`: merged Klipper jerk admission uses default is_approx.

#[test]
fn within_native_tolerance_suppresses_merged_jerk() {
    crate::runner::native_stream_case::check(
        (
            "merged-jerk-tolerance",
            &[
                "GCodeWriter.cpp:333-338::set_accel_and_jerk",
                "libslic3r.h:52,306-309::is_approx",
            ],
        ),
        "within-native-tolerance",
        include_bytes!(
            "../../../../tests/parity/merged-jerk-tolerance/within-native-tolerance.3mf"
        ),
        "e407c7c7ea2f19cfd1436c7406b1d8f31b1fd72f1a776c60e814bfb20a276f60",
        (false, |reference, _| {
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
                .flat_map(str::split_ascii_whitespace)
                .filter_map(|word| word.strip_prefix("SQUARE_CORNER_VELOCITY="))
                .map(|word| word.parse().unwrap())
                .collect();
            assert!(
                !values.is_empty(),
                "Native must exercise square-corner velocity"
            );
            assert_eq!(values.windows(2).any(|pair| pair[0] != pair[1]), false);
        }),
    );
}

#[test]
fn equal_control_stays_byte_equal() {
    crate::runner::native_stream_case::check(
        (
            "merged-jerk-tolerance",
            &[
                "GCodeWriter.cpp:333-338::set_accel_and_jerk",
                "libslic3r.h:52,306-309::is_approx",
            ],
        ),
        "equal-control",
        include_bytes!("../../../../tests/parity/merged-jerk-tolerance/equal-control.3mf"),
        "173b2cd1e4d549de65953512f9d8d3d853f3753f68ff0546b0be63368987a78b",
        (false, |reference, _| {
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
                .flat_map(str::split_ascii_whitespace)
                .filter_map(|word| word.strip_prefix("SQUARE_CORNER_VELOCITY="))
                .map(|word| word.parse().unwrap())
                .collect();
            assert!(
                !values.is_empty(),
                "Native must exercise square-corner velocity"
            );
            assert_eq!(values.windows(2).any(|pair| pair[0] != pair[1]), false);
        }),
    );
}

#[test]
fn within_both_control_stays_byte_equal() {
    crate::runner::native_stream_case::check(
        (
            "merged-jerk-tolerance",
            &[
                "GCodeWriter.cpp:333-338::set_accel_and_jerk",
                "libslic3r.h:52,306-309::is_approx",
            ],
        ),
        "within-both-tolerances",
        include_bytes!("../../../../tests/parity/merged-jerk-tolerance/within-both-tolerances.3mf"),
        "df9e6effe76a115996017892f4b488764add7073744619f7d69a4b7adabd9d4b",
        (false, |reference, _| {
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
                .flat_map(str::split_ascii_whitespace)
                .filter_map(|word| word.strip_prefix("SQUARE_CORNER_VELOCITY="))
                .map(|word| word.parse().unwrap())
                .collect();
            assert!(
                !values.is_empty(),
                "Native must exercise square-corner velocity"
            );
            assert_eq!(values.windows(2).any(|pair| pair[0] != pair[1]), false);
        }),
    );
}

#[test]
fn above_native_tolerance_admits_merged_jerk() {
    crate::runner::native_stream_case::check(
        (
            "merged-jerk-tolerance",
            &[
                "GCodeWriter.cpp:333-338::set_accel_and_jerk",
                "libslic3r.h:52,306-309::is_approx",
            ],
        ),
        "above-native-tolerance-clean",
        include_bytes!(
            "../../../../tests/parity/merged-jerk-tolerance/above-native-tolerance-clean.3mf"
        ),
        "53ddea7da1ffd0262c4a04d9c83913adc0b46167349950732b4613099a584991",
        (false, |reference, _| {
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
                .flat_map(str::split_ascii_whitespace)
                .filter_map(|word| word.strip_prefix("SQUARE_CORNER_VELOCITY="))
                .map(|word| word.parse().unwrap())
                .collect();
            assert!(
                !values.is_empty(),
                "Native must exercise square-corner velocity"
            );
            assert_eq!(values.windows(2).any(|pair| pair[0] != pair[1]), true);
        }),
    );
}
