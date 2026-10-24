//! `GCodeWriter.cpp:315-337`: merged square-corner velocity streams a double.

#[test]
fn fractional_merged_jerk_retains_ostream_precision() {
    let project =
        include_bytes!("../../../../tests/parity/merged-jerk-precision/precision-target.3mf");
    crate::runner::native_stream_case::check(
        (
            "merged-jerk-precision",
            &[
                "GCodeWriter.cpp:315-337::set_accel_and_jerk",
                "motion/format.rs::ostream",
            ],
        ),
        "precision-target",
        project,
        "8b1a2d4927db6ce127d8f8392fab228a04485922a0db741de43d5a93d79fdee3",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["inner_wall_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
                .flat_map(str::split_ascii_whitespace)
                .filter_map(|word| word.strip_prefix("SQUARE_CORNER_VELOCITY="))
                .map(|word| word.parse().unwrap())
                .collect();
            assert!(
                values.contains(&expected),
                "Native serialization retains the saved inner jerk"
            );
        }),
    );
}

#[test]
fn equal_merged_jerk_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/merged-jerk-precision/equal-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "merged-jerk-precision",
            &[
                "GCodeWriter.cpp:315-337::set_accel_and_jerk",
                "motion/format.rs::ostream",
            ],
        ),
        "equal-control",
        project,
        "173b2cd1e4d549de65953512f9d8d3d853f3753f68ff0546b0be63368987a78b",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["inner_wall_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
                .flat_map(str::split_ascii_whitespace)
                .filter_map(|word| word.strip_prefix("SQUARE_CORNER_VELOCITY="))
                .map(|word| word.parse().unwrap())
                .collect();
            assert!(
                values.contains(&expected),
                "Native serialization retains the saved inner jerk"
            );
        }),
    );
}

#[test]
fn exact_milli_merged_jerk_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/merged-jerk-precision/exact-milli-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "merged-jerk-precision",
            &[
                "GCodeWriter.cpp:315-337::set_accel_and_jerk",
                "motion/format.rs::ostream",
            ],
        ),
        "exact-milli-control",
        project,
        "53ddea7da1ffd0262c4a04d9c83913adc0b46167349950732b4613099a584991",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["inner_wall_jerk"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let values: Vec<f64> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
                .flat_map(str::split_ascii_whitespace)
                .filter_map(|word| word.strip_prefix("SQUARE_CORNER_VELOCITY="))
                .map(|word| word.parse().unwrap())
                .collect();
            assert!(
                values.contains(&expected),
                "Native serialization retains the saved inner jerk"
            );
        }),
    );
}
