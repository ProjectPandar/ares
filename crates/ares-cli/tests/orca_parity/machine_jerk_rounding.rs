//! `GCodeWriter.cpp:32-56`: flavor-gated XY machine jerk caches use lrint.

#[test]
fn fractional_machine_x_limit_uses_rounded_writer_cache() {
    let project =
        include_bytes!("../../../../tests/parity/machine-jerk-rounding/fractional-x-limit.3mf");
    crate::runner::native_stream_case::check(
        (
            "machine-jerk-rounding",
            &[
                "GCodeWriter.cpp:32-56::apply_print_config",
                "GCodeWriter.cpp:289-293::set_jerk_xy",
            ],
        ),
        "fractional-x-limit",
        project,
        "4ea82ec44a0d1c7527800e666c7a294c71b717fb90a1c361d09f82dff00241d7",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let raw: f64 = settings["machine_max_jerk_x"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let (header, body) = reference.native.split_once(";TYPE:Custom\n").unwrap();
            let header_x: f64 = header
                .lines()
                .find_map(|line| line.strip_prefix("M205 X"))
                .unwrap()
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(header_x, raw, "Native header retains raw machine limit");
            let xs: Vec<f64> = body
                .lines()
                .filter_map(|line| line.strip_prefix("M205 X"))
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
                !xs.is_empty(),
                "Native must exercise active body jerk clamps"
            );
            assert!(xs.iter().all(|value| *value == raw.round_ties_even()));
        }),
    );
}

#[test]
fn integer_machine_limit_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/machine-jerk-rounding/integer-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "machine-jerk-rounding",
            &[
                "GCodeWriter.cpp:32-56::apply_print_config",
                "GCodeWriter.cpp:289-293::set_jerk_xy",
            ],
        ),
        "integer-control",
        project,
        "fc0fba0a17069c48a0ddcd6bd332025db4702f659d71cbd9c6a7f5a087990132",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let raw: f64 = settings["machine_max_jerk_x"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let (header, body) = reference.native.split_once(";TYPE:Custom\n").unwrap();
            let header_x: f64 = header
                .lines()
                .find_map(|line| line.strip_prefix("M205 X"))
                .unwrap()
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(header_x, raw, "Native header retains raw machine limit");
            let xs: Vec<f64> = body
                .lines()
                .filter_map(|line| line.strip_prefix("M205 X"))
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
                !xs.is_empty(),
                "Native must exercise active body jerk clamps"
            );
            assert!(xs.iter().all(|value| *value == raw.round_ties_even()));
        }),
    );
}

#[test]
fn inactive_jerk_control_preserves_raw_header() {
    let project =
        include_bytes!("../../../../tests/parity/machine-jerk-rounding/inactive-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "machine-jerk-rounding",
            &[
                "GCodeWriter.cpp:32-56::apply_print_config",
                "GCodeWriter.cpp:289-293::set_jerk_xy",
            ],
        ),
        "inactive-control",
        project,
        "6b3c9d972c96f04cc849e3da9d67ff06679146d94b53e39819fb16916be4d152",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let raw: f64 = settings["machine_max_jerk_x"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let (header, body) = reference.native.split_once(";TYPE:Custom\n").unwrap();
            let header_x: f64 = header
                .lines()
                .find_map(|line| line.strip_prefix("M205 X"))
                .unwrap()
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(header_x, raw, "Native header retains raw machine limit");
            let xs: Vec<f64> = body
                .lines()
                .filter_map(|line| line.strip_prefix("M205 X"))
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
                xs.is_empty(),
                "Native inactive control has no body jerk command"
            );
        }),
    );
}
