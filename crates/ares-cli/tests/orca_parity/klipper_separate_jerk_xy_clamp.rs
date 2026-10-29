//! `GCodeWriter.cpp:263-270`: separate Klipper scalar clamps X then Y.

#[test]
fn lower_y_limit_clamps_separate_klipper_jerk() {
    let project =
        include_bytes!("../../../../tests/parity/klipper-separate-jerk-xy-clamp/lower-y-limit.3mf");
    crate::runner::native_stream_case::check(
        (
            "klipper-separate-jerk-xy-clamp",
            &["GCodeWriter.cpp:263-270::set_jerk_xy"],
        ),
        "lower-y-limit",
        project,
        "744139aab80a6f3ead834cdd93c47190bfab2e717c1e8ecd7a4da47e306eb542",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let mut expected: f64 = settings["initial_layer_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            for axis in ["machine_max_jerk_x", "machine_max_jerk_y"] {
                let limit: f64 = settings[axis][0]
                    .as_str()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    .round_ties_even();
                if limit > 0.0 {
                    expected = expected.min(limit);
                }
            }
            let actual: f64 = reference
                .native
                .lines()
                .filter(|line| !line.contains(" ACCEL="))
                .find_map(|line| line.strip_prefix("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY="))
                .unwrap()
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native initial separate jerk honors both positive limits"
            );
        }),
    );
}

#[test]
fn equal_xy_limits_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/klipper-separate-jerk-xy-clamp/equal-limits-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "klipper-separate-jerk-xy-clamp",
            &["GCodeWriter.cpp:263-270::set_jerk_xy"],
        ),
        "equal-limits-control",
        project,
        "3a8b3f3fbbcbdc9b7b5eb1ee13e5e08533ae8e54d60fd814be46ff45b1c02114",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let mut expected: f64 = settings["initial_layer_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            for axis in ["machine_max_jerk_x", "machine_max_jerk_y"] {
                let limit: f64 = settings[axis][0]
                    .as_str()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    .round_ties_even();
                if limit > 0.0 {
                    expected = expected.min(limit);
                }
            }
            let actual: f64 = reference
                .native
                .lines()
                .filter(|line| !line.contains(" ACCEL="))
                .find_map(|line| line.strip_prefix("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY="))
                .unwrap()
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native initial separate jerk honors both positive limits"
            );
        }),
    );
}

#[test]
fn lower_x_limit_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/klipper-separate-jerk-xy-clamp/lower-x-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "klipper-separate-jerk-xy-clamp",
            &["GCodeWriter.cpp:263-270::set_jerk_xy"],
        ),
        "lower-x-control",
        project,
        "289bbdd4c40fc49f1c5eafcceaa890de41ddb01d19fe66a2b1699a6ea7397032",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let mut expected: f64 = settings["initial_layer_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            for axis in ["machine_max_jerk_x", "machine_max_jerk_y"] {
                let limit: f64 = settings[axis][0]
                    .as_str()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    .round_ties_even();
                if limit > 0.0 {
                    expected = expected.min(limit);
                }
            }
            let actual: f64 = reference
                .native
                .lines()
                .filter(|line| !line.contains(" ACCEL="))
                .find_map(|line| line.strip_prefix("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY="))
                .unwrap()
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native initial separate jerk honors both positive limits"
            );
        }),
    );
}
