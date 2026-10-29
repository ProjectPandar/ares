//! `GCodeWriter.cpp:241,324`: ACCEL_TO_DECEL streams a double in both writers.

#[test]
fn fractional_factor_retains_ostream_precision() {
    let project =
        include_bytes!("../../../../tests/parity/accel-to-decel-precision/fractional-factor.3mf");
    crate::runner::native_stream_case::check(
        (
            "accel-to-decel-precision",
            &[
                "GCodeWriter.cpp:216-249::set_acceleration",
                "GCodeWriter.cpp:311-338::set_accel_and_jerk",
            ],
        ),
        "fractional-factor",
        project,
        "56cb137c5fdf3db5899ce19146a10bbd2b263f6527e7ebff2fd70881fdbf0927",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let factor: f64 = settings["accel_to_decel_factor"]
                .as_str()
                .unwrap()
                .trim_end_matches('%')
                .parse()
                .unwrap();
            let mut accelerations = std::collections::BTreeSet::new();
            for line in reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
            {
                let Some(decel) = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix("ACCEL_TO_DECEL="))
                else {
                    continue;
                };
                let accel: u32 = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix("ACCEL="))
                    .unwrap()
                    .parse()
                    .unwrap();
                let expected: f64 = format!("{:.5e}", f64::from(accel) * factor / 100.0)
                    .parse()
                    .unwrap();
                let actual: f64 = decel.parse().unwrap();
                assert_eq!(
                    actual, expected,
                    "Native uses six significant digits after double arithmetic"
                );
                accelerations.insert(accel);
            }
            assert!(
                accelerations.len() > 1,
                "Native must exercise distinct acceleration transitions"
            );
        }),
    );
}

#[test]
fn half_factor_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/accel-to-decel-precision/factor-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "accel-to-decel-precision",
            &[
                "GCodeWriter.cpp:216-249::set_acceleration",
                "GCodeWriter.cpp:311-338::set_accel_and_jerk",
            ],
        ),
        "factor-control",
        project,
        "a1c7580b4a1e596fa8fdd72ba2301d95b6b6539a2cd74e9ee612f6d930c6091d",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let factor: f64 = settings["accel_to_decel_factor"]
                .as_str()
                .unwrap()
                .trim_end_matches('%')
                .parse()
                .unwrap();
            let mut accelerations = std::collections::BTreeSet::new();
            for line in reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
            {
                let Some(decel) = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix("ACCEL_TO_DECEL="))
                else {
                    continue;
                };
                let accel: u32 = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix("ACCEL="))
                    .unwrap()
                    .parse()
                    .unwrap();
                let expected: f64 = format!("{:.5e}", f64::from(accel) * factor / 100.0)
                    .parse()
                    .unwrap();
                let actual: f64 = decel.parse().unwrap();
                assert_eq!(
                    actual, expected,
                    "Native uses six significant digits after double arithmetic"
                );
                accelerations.insert(accel);
            }
            assert!(
                accelerations.len() > 1,
                "Native must exercise distinct acceleration transitions"
            );
        }),
    );
}

#[test]
fn exact_decimal_factor_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/accel-to-decel-precision/exact-decimal-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "accel-to-decel-precision",
            &[
                "GCodeWriter.cpp:216-249::set_acceleration",
                "GCodeWriter.cpp:311-338::set_accel_and_jerk",
            ],
        ),
        "exact-decimal-control",
        project,
        "c73b59deb59296e87c8863ad3048e401876079791c2f282eb273ca1134f577ec",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let factor: f64 = settings["accel_to_decel_factor"]
                .as_str()
                .unwrap()
                .trim_end_matches('%')
                .parse()
                .unwrap();
            let mut accelerations = std::collections::BTreeSet::new();
            for line in reference
                .native
                .lines()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT "))
            {
                let Some(decel) = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix("ACCEL_TO_DECEL="))
                else {
                    continue;
                };
                let accel: u32 = line
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix("ACCEL="))
                    .unwrap()
                    .parse()
                    .unwrap();
                let expected: f64 = format!("{:.5e}", f64::from(accel) * factor / 100.0)
                    .parse()
                    .unwrap();
                let actual: f64 = decel.parse().unwrap();
                assert_eq!(
                    actual, expected,
                    "Native uses six significant digits after double arithmetic"
                );
                accelerations.insert(accel);
            }
            assert!(
                accelerations.len() > 1,
                "Native must exercise distinct acceleration transitions"
            );
        }),
    );
}
