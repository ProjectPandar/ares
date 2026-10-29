//! `GCodeWriter.cpp:262-270`: separate Klipper jerk double serialization.

#[test]
fn fractional_separate_klipper_jerk_retains_precision() {
    let project = include_bytes!(
        "../../../../tests/parity/klipper-separate-jerk-precision/fractional-separate-jerk.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "klipper-separate-jerk-precision",
            &["GCodeWriter.cpp:262-270::set_jerk_xy"],
        ),
        "fractional-separate-jerk",
        project,
        "68ec6624b778a08282b7767a1cac831bd27db075c08cd59249dc8734b7058f12",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["initial_layer_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| !line.contains(" ACCEL="))
                .filter_map(|line| line.strip_prefix("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY="))
                .map(|words| {
                    words
                        .split_ascii_whitespace()
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap()
                })
                .collect();
            assert!(
                values.contains(&expected),
                "Native separate command retains input initial jerk"
            );
        }),
    );
}

#[test]
fn integer_separate_klipper_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/klipper-separate-jerk-precision/integer-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "klipper-separate-jerk-precision",
            &["GCodeWriter.cpp:262-270::set_jerk_xy"],
        ),
        "integer-control",
        project,
        "3a8b3f3fbbcbdc9b7b5eb1ee13e5e08533ae8e54d60fd814be46ff45b1c02114",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["initial_layer_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| !line.contains(" ACCEL="))
                .filter_map(|line| line.strip_prefix("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY="))
                .map(|words| {
                    words
                        .split_ascii_whitespace()
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap()
                })
                .collect();
            assert!(
                values.contains(&expected),
                "Native separate command retains input initial jerk"
            );
        }),
    );
}

#[test]
fn milli_separate_klipper_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/klipper-separate-jerk-precision/exact-milli-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "klipper-separate-jerk-precision",
            &["GCodeWriter.cpp:262-270::set_jerk_xy"],
        ),
        "exact-milli-control",
        project,
        "d07cef12a7b998feb5801acb41eb3c68da13cbb5517916cb105270b8f7f2636a",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["initial_layer_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| !line.contains(" ACCEL="))
                .filter_map(|line| line.strip_prefix("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY="))
                .map(|words| {
                    words
                        .split_ascii_whitespace()
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap()
                })
                .collect();
            assert!(
                values.contains(&expected),
                "Native separate command retains input initial jerk"
            );
        }),
    );
}
