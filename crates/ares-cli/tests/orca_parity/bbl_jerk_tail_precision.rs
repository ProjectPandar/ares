//! `GCodeWriter.cpp:298-299`: BBL jerk Z/E suffix uses defaultfloat precision two.

#[test]
fn subunit_bbl_z_jerk_keeps_defaultfloat_prefix() {
    let project =
        include_bytes!("../../../../tests/parity/bbl-jerk-tail-precision/fractional-z-exact.3mf");
    crate::runner::native_stream_case::check(
        (
            "bbl-jerk-tail-precision",
            &["GCodeWriter.cpp:298-299::set_jerk_xy"],
        ),
        "fractional-z-exact",
        project,
        "5eb57eb85313786b8b3f19cf8591771c2d79aeda963e571c619a20ba2fb18a52",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
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
            for (key, prefix) in [("machine_max_jerk_z", "Z"), ("machine_max_jerk_e", "E")] {
                let raw: f64 = settings[key][0].as_str().unwrap().parse().unwrap();
                let expected = raw.to_string();
                assert!(
                    commands.iter().all(|line| line
                        .split_ascii_whitespace()
                        .find_map(|word| word.strip_prefix(prefix))
                        == Some(expected.as_str())),
                    "Native clean tail text retains the saved two-significant-digit value"
                );
            }
        }),
    );
}

#[test]
fn stock_bbl_jerk_tail_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/bbl-jerk-tail-precision/stock-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "bbl-jerk-tail-precision",
            &["GCodeWriter.cpp:298-299::set_jerk_xy"],
        ),
        "stock-control",
        project,
        "8218407a4f814505bf6df4f4c82b0c67a39c73acfaf938da72ba5144f50fe32f",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
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
            for (key, prefix) in [("machine_max_jerk_z", "Z"), ("machine_max_jerk_e", "E")] {
                let raw: f64 = settings[key][0].as_str().unwrap().parse().unwrap();
                let expected = raw.to_string();
                assert!(
                    commands.iter().all(|line| line
                        .split_ascii_whitespace()
                        .find_map(|word| word.strip_prefix(prefix))
                        == Some(expected.as_str())),
                    "Native clean tail text retains the saved two-significant-digit value"
                );
            }
        }),
    );
}

#[test]
fn positive_bbl_z_jerk_control_stays_byte_equal() {
    let project =
        include_bytes!("../../../../tests/parity/bbl-jerk-tail-precision/positive-z-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "bbl-jerk-tail-precision",
            &["GCodeWriter.cpp:298-299::set_jerk_xy"],
        ),
        "positive-z-control",
        project,
        "70db0f1afd38380c28f93b2d435b01120d49cb74a46cf3925c10ed6ddcf03afe",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
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
            for (key, prefix) in [("machine_max_jerk_z", "Z"), ("machine_max_jerk_e", "E")] {
                let raw: f64 = settings[key][0].as_str().unwrap().parse().unwrap();
                let expected = raw.to_string();
                assert!(
                    commands.iter().all(|line| line
                        .split_ascii_whitespace()
                        .find_map(|word| word.strip_prefix(prefix))
                        == Some(expected.as_str())),
                    "Native clean tail text retains the saved two-significant-digit value"
                );
            }
        }),
    );
}
