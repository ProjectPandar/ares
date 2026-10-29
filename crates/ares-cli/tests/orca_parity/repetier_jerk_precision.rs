//! `GCodeWriter.cpp:272-285`: Repetier M207 streams the double jerk.

#[test]
fn fractional_repetier_jerk_retains_ostream_precision() {
    let project =
        include_bytes!("../../../../tests/parity/repetier-jerk-precision/fractional-jerk.3mf");
    crate::runner::native_stream_case::check(
        (
            "repetier-jerk-precision",
            &["GCodeWriter.cpp:262,272-285::set_jerk_xy"],
        ),
        "fractional-jerk",
        project,
        "1029964bc3ef6b4457d215249a85db6c75692f7dcdc22cb77ff307338f420076",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["default_jerk"].as_str().unwrap().parse().unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter_map(|line| line.strip_prefix("M207 X"))
                .collect();
            assert!(commands.iter().any(|words| {
                words
                    .split_ascii_whitespace()
                    .next()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    == expected
            }));
        }),
    );
}

#[test]
fn integer_repetier_jerk_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/repetier-jerk-precision/integer-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "repetier-jerk-precision",
            &["GCodeWriter.cpp:262,272-285::set_jerk_xy"],
        ),
        "integer-control",
        project,
        "dd456257226ef4563c4742764fee803e47fc459568f31da39f59413441e6d626",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["default_jerk"].as_str().unwrap().parse().unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter_map(|line| line.strip_prefix("M207 X"))
                .collect();
            assert!(commands.iter().any(|words| {
                words
                    .split_ascii_whitespace()
                    .next()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    == expected
            }));
        }),
    );
}

#[test]
fn exact_milli_repetier_jerk_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/repetier-jerk-precision/exact-milli-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "repetier-jerk-precision",
            &["GCodeWriter.cpp:262,272-285::set_jerk_xy"],
        ),
        "exact-milli-control",
        project,
        "ea68cf109f4abd9ff14966be8eeb9627fc179d258cb11ab4991b00cd1b274201",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["default_jerk"].as_str().unwrap().parse().unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter_map(|line| line.strip_prefix("M207 X"))
                .collect();
            assert!(commands.iter().any(|words| {
                words
                    .split_ascii_whitespace()
                    .next()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    == expected
            }));
        }),
    );
}
