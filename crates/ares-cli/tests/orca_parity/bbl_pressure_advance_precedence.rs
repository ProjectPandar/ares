//! `GCodeWriter.cpp:370-392`: BBL PA command selection precedes flavor.

#[test]
fn bbl_pressure_advance_precedes_repetier_flavor() {
    let project = include_bytes!(
        "../../../../tests/parity/bbl-pressure-advance-precedence/bbl-repetier-pa.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "bbl-pressure-advance-precedence",
            &["GCodeWriter.cpp:370-392::set_pressure_advance"],
        ),
        "bbl-repetier-pa",
        project,
        "f90235bc55fec8361f0ebe8ecf7b78239229e6dca766d7a647d794c55a295693",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            assert!(
                settings["printer_model"]
                    .as_str()
                    .unwrap()
                    .starts_with("Bambu Lab")
            );
            let enabled = settings["enable_pressure_advance"][0] == "1";
            let value: f64 = settings["pressure_advance"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.ends_with("; Override pressure advance value"))
                .collect();
            if enabled {
                let expected = format!("M900 K{value} L1000 M10 ; Override pressure advance value");
                assert!(
                    !commands.is_empty(),
                    "Native must emit enabled BBL PA overrides"
                );
                assert!(
                    commands.iter().all(|line| *line == expected),
                    "Native BBL PA must precede the saved flavor"
                );
            } else {
                assert!(
                    commands.is_empty(),
                    "Native must not generate disabled PA overrides"
                );
            }
        }),
    );
}

#[test]
fn stock_bbl_pressure_advance_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/bbl-pressure-advance-precedence/bbl-stock-pa-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "bbl-pressure-advance-precedence",
            &["GCodeWriter.cpp:370-392::set_pressure_advance"],
        ),
        "bbl-stock-pa-control",
        project,
        "1d56a96dfa6bb2d86ef8a2d29be12ee0ea395e25d96172ee9a07a87f70456b14",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            assert!(
                settings["printer_model"]
                    .as_str()
                    .unwrap()
                    .starts_with("Bambu Lab")
            );
            let enabled = settings["enable_pressure_advance"][0] == "1";
            let value: f64 = settings["pressure_advance"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.ends_with("; Override pressure advance value"))
                .collect();
            if enabled {
                let expected = format!("M900 K{value} L1000 M10 ; Override pressure advance value");
                assert!(
                    !commands.is_empty(),
                    "Native must emit enabled BBL PA overrides"
                );
                assert!(
                    commands.iter().all(|line| *line == expected),
                    "Native BBL PA must precede the saved flavor"
                );
            } else {
                assert!(
                    commands.is_empty(),
                    "Native must not generate disabled PA overrides"
                );
            }
        }),
    );
}

#[test]
fn disabled_bbl_repetier_pressure_advance_control_stays_silent() {
    let project = include_bytes!(
        "../../../../tests/parity/bbl-pressure-advance-precedence/bbl-repetier-pa-disabled-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "bbl-pressure-advance-precedence",
            &["GCodeWriter.cpp:370-392::set_pressure_advance"],
        ),
        "bbl-repetier-pa-disabled-control",
        project,
        "92dbe0333f559c08a88f125018bf719dfcf2d2588b1cff98e7b059a634f58204",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            assert!(
                settings["printer_model"]
                    .as_str()
                    .unwrap()
                    .starts_with("Bambu Lab")
            );
            let enabled = settings["enable_pressure_advance"][0] == "1";
            let value: f64 = settings["pressure_advance"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| line.ends_with("; Override pressure advance value"))
                .collect();
            if enabled {
                let expected = format!("M900 K{value} L1000 M10 ; Override pressure advance value");
                assert!(
                    !commands.is_empty(),
                    "Native must emit enabled BBL PA overrides"
                );
                assert!(
                    commands.iter().all(|line| *line == expected),
                    "Native BBL PA must precede the saved flavor"
                );
            } else {
                assert!(
                    commands.is_empty(),
                    "Native must not generate disabled PA overrides"
                );
            }
        }),
    );
}
