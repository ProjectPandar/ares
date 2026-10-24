//! `GCodeWriter.cpp:298-299`: streamed BBL Z jerk retains negative zero.

#[test]
fn negative_zero_bbl_z_jerk_retains_sign() {
    let project =
        include_bytes!("../../../../tests/parity/bbl-jerk-tail-signed-zero/negative-zero-z.3mf");
    crate::runner::native_stream_case::check(
        (
            "bbl-jerk-tail-signed-zero",
            &[
                "GCodeWriter.cpp:58-59::apply_print_config",
                "GCodeWriter.cpp:298-299::set_jerk_xy",
            ],
        ),
        "negative-zero-z",
        project,
        "c9eb20cb0b3390c71180e76854c80c9f81357e24e40abfba652f6582584f4891",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["machine_max_jerk_z"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("M205 X") && !line.contains("sets the jerk limits"))
                .collect();
            assert!(
                !commands.is_empty(),
                "Native must emit BBL body jerk commands"
            );
            for line in commands {
                let actual: f64 = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('Z'))
                    .unwrap()
                    .parse()
                    .unwrap();
                assert_eq!(
                    actual, expected,
                    "Native Z jerk must retain the saved numeric value"
                );
                if expected == 0.0 {
                    assert_eq!(actual.classify(), std::num::FpCategory::Zero);
                    assert_eq!(
                        actual.is_sign_negative(),
                        expected.is_sign_negative(),
                        "Numeric zero equality alone must not erase the saved sign"
                    );
                }
            }
        }),
    );
}

#[test]
fn stock_bbl_z_jerk_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/bbl-jerk-tail-signed-zero/stock-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "bbl-jerk-tail-signed-zero",
            &[
                "GCodeWriter.cpp:58-59::apply_print_config",
                "GCodeWriter.cpp:298-299::set_jerk_xy",
            ],
        ),
        "stock-control",
        project,
        "6cb522350420360807f04e3e920a71c71286a92583f2d513a8def2eec5b2c975",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["machine_max_jerk_z"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("M205 X") && !line.contains("sets the jerk limits"))
                .collect();
            assert!(
                !commands.is_empty(),
                "Native must emit BBL body jerk commands"
            );
            for line in commands {
                let actual: f64 = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('Z'))
                    .unwrap()
                    .parse()
                    .unwrap();
                assert_eq!(
                    actual, expected,
                    "Native Z jerk must retain the saved numeric value"
                );
                if expected == 0.0 {
                    assert_eq!(actual.classify(), std::num::FpCategory::Zero);
                    assert_eq!(
                        actual.is_sign_negative(),
                        expected.is_sign_negative(),
                        "Numeric zero equality alone must not erase the saved sign"
                    );
                }
            }
        }),
    );
}

#[test]
fn positive_zero_bbl_z_jerk_control_stays_positive() {
    let project = include_bytes!(
        "../../../../tests/parity/bbl-jerk-tail-signed-zero/positive-zero-z-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "bbl-jerk-tail-signed-zero",
            &[
                "GCodeWriter.cpp:58-59::apply_print_config",
                "GCodeWriter.cpp:298-299::set_jerk_xy",
            ],
        ),
        "positive-zero-z-control",
        project,
        "aa4f1d49d031216cc1fc3c325bc9562f0c3527e7499ac687336e1a6ce8421ce7",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let expected: f64 = settings["machine_max_jerk_z"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.starts_with("M205 X") && !line.contains("sets the jerk limits"))
                .collect();
            assert!(
                !commands.is_empty(),
                "Native must emit BBL body jerk commands"
            );
            for line in commands {
                let actual: f64 = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('Z'))
                    .unwrap()
                    .parse()
                    .unwrap();
                assert_eq!(
                    actual, expected,
                    "Native Z jerk must retain the saved numeric value"
                );
                if expected == 0.0 {
                    assert_eq!(actual.classify(), std::num::FpCategory::Zero);
                    assert_eq!(
                        actual.is_sign_negative(),
                        expected.is_sign_negative(),
                        "Numeric zero equality alone must not erase the saved sign"
                    );
                }
            }
        }),
    );
}
