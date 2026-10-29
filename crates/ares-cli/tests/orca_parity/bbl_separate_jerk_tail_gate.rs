//! `GCodeWriter.cpp:298-299`: the BBL jerk tail follows all flavor prefixes.

#[test]
fn bbl_repetier_jerk_retains_ze_tail() {
    let project = include_bytes!(
        "../../../../tests/parity/bbl-separate-jerk-tail-gate/bbl-repetier-tail.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "bbl-separate-jerk-tail-gate",
            &["GCodeWriter.cpp:298-299::set_jerk_xy"],
        ),
        "bbl-repetier-tail",
        project,
        "92dbe0333f559c08a88f125018bf719dfcf2d2588b1cff98e7b059a634f58204",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let bbl = settings["printer_model"]
                .as_str()
                .unwrap()
                .starts_with("Bambu Lab");
            let prefix = if settings["gcode_flavor"] == "repetier" {
                "M207 X"
            } else {
                "M205 X"
            };
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.starts_with(prefix) && !line.contains("sets the jerk limits"))
                .collect();
            assert!(
                !commands.is_empty(),
                "Native must emit body jerk commands for the saved flavor"
            );
            for (key, axis) in [("machine_max_jerk_z", "Z"), ("machine_max_jerk_e", "E")] {
                let value: f64 = settings[key][0].as_str().unwrap().parse().unwrap();
                let expected = bbl.then(|| value.to_string());
                assert!(
                    commands.iter().all(|line| line
                        .split_ascii_whitespace()
                        .find_map(|word| word.strip_prefix(axis))
                        == expected.as_deref()),
                    "Native BBL suffix admission must be independent of the saved flavor"
                );
            }
        }),
    );
}

#[test]
fn stock_bbl_jerk_tail_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/bbl-separate-jerk-tail-gate/stock-bbl-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "bbl-separate-jerk-tail-gate",
            &["GCodeWriter.cpp:298-299::set_jerk_xy"],
        ),
        "stock-bbl-control",
        project,
        "6cb522350420360807f04e3e920a71c71286a92583f2d513a8def2eec5b2c975",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let bbl = settings["printer_model"]
                .as_str()
                .unwrap()
                .starts_with("Bambu Lab");
            let prefix = if settings["gcode_flavor"] == "repetier" {
                "M207 X"
            } else {
                "M205 X"
            };
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.starts_with(prefix) && !line.contains("sets the jerk limits"))
                .collect();
            assert!(
                !commands.is_empty(),
                "Native must emit body jerk commands for the saved flavor"
            );
            for (key, axis) in [("machine_max_jerk_z", "Z"), ("machine_max_jerk_e", "E")] {
                let value: f64 = settings[key][0].as_str().unwrap().parse().unwrap();
                let expected = bbl.then(|| value.to_string());
                assert!(
                    commands.iter().all(|line| line
                        .split_ascii_whitespace()
                        .find_map(|word| word.strip_prefix(axis))
                        == expected.as_deref()),
                    "Native BBL suffix admission must be independent of the saved flavor"
                );
            }
        }),
    );
}

#[test]
fn non_bbl_repetier_jerk_control_has_no_tail() {
    let project = include_bytes!(
        "../../../../tests/parity/bbl-separate-jerk-tail-gate/ender3-repetier-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "bbl-separate-jerk-tail-gate",
            &["GCodeWriter.cpp:298-299::set_jerk_xy"],
        ),
        "ender3-repetier-control",
        project,
        "bf34d2e7ccfc2e33b1311cab464d1f57f7883da5165a4f643da309d98df38254",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let bbl = settings["printer_model"]
                .as_str()
                .unwrap()
                .starts_with("Bambu Lab");
            let prefix = if settings["gcode_flavor"] == "repetier" {
                "M207 X"
            } else {
                "M205 X"
            };
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.starts_with(prefix) && !line.contains("sets the jerk limits"))
                .collect();
            assert!(
                !commands.is_empty(),
                "Native must emit body jerk commands for the saved flavor"
            );
            for (key, axis) in [("machine_max_jerk_z", "Z"), ("machine_max_jerk_e", "E")] {
                let value: f64 = settings[key][0].as_str().unwrap().parse().unwrap();
                let expected = bbl.then(|| value.to_string());
                assert!(
                    commands.iter().all(|line| line
                        .split_ascii_whitespace()
                        .find_map(|word| word.strip_prefix(axis))
                        == expected.as_deref()),
                    "Native BBL suffix admission must be independent of the saved flavor"
                );
            }
        }),
    );
}
