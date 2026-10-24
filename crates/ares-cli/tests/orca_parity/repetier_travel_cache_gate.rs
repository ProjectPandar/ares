//! `GCodeWriter.cpp:49-52`: travel cache requires both source predicates.

#[test]
fn repetier_travel_ignores_inadmissible_machine_cap() {
    let project = include_bytes!(
        "../../../../tests/parity/repetier-travel-cache-gate/repetier-over-limit.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "repetier-travel-cache-gate",
            &["GCodeWriter.cpp:49-52::apply_print_config"],
        ),
        "repetier-over-limit",
        project,
        "a1bbe4c18e06b709f580a67fe8bec437ca1d080f1b70912075252229f0d53b65",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let requested: f64 = settings["travel_acceleration"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let cap: f64 = settings["machine_max_acceleration_travel"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let repetier = settings["gcode_flavor"] == "repetier";
            let expected = if repetier {
                requested.round()
            } else {
                requested.round().min(cap.round())
            } as u32;
            let prefix = if repetier { "M202 X" } else { "M204 T" };
            let actual: u32 = reference
                .native
                .lines()
                .find_map(|line| line.strip_prefix(prefix))
                .expect("Native must emit a body travel acceleration command")
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native travel commands must retain separate support while applying only admitted machine caps"
            );
        }),
    );
}

#[test]
fn repetier_below_limit_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/repetier-travel-cache-gate/repetier-below-limit-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "repetier-travel-cache-gate",
            &["GCodeWriter.cpp:49-52::apply_print_config"],
        ),
        "repetier-below-limit-control",
        project,
        "97495393518915bbd5285bdd90785745f03b1ef50cc625d5251daeaf0fd64d07",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let requested: f64 = settings["travel_acceleration"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let cap: f64 = settings["machine_max_acceleration_travel"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let repetier = settings["gcode_flavor"] == "repetier";
            let expected = if repetier {
                requested.round()
            } else {
                requested.round().min(cap.round())
            } as u32;
            let prefix = if repetier { "M202 X" } else { "M204 T" };
            let actual: u32 = reference
                .native
                .lines()
                .find_map(|line| line.strip_prefix(prefix))
                .expect("Native must emit a body travel acceleration command")
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native travel commands must retain separate support while applying only admitted machine caps"
            );
        }),
    );
}

#[test]
fn marlin2_travel_limit_control_remains_capped() {
    let project = include_bytes!(
        "../../../../tests/parity/repetier-travel-cache-gate/marlin2-limit-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "repetier-travel-cache-gate",
            &["GCodeWriter.cpp:49-52::apply_print_config"],
        ),
        "marlin2-limit-control",
        project,
        "2b232b6735ba52645223c1adbf9734320efccbbd5f5f347df7458cb0c1317134",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let requested: f64 = settings["travel_acceleration"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let cap: f64 = settings["machine_max_acceleration_travel"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let repetier = settings["gcode_flavor"] == "repetier";
            let expected = if repetier {
                requested.round()
            } else {
                requested.round().min(cap.round())
            } as u32;
            let prefix = if repetier { "M202 X" } else { "M204 T" };
            let actual: u32 = reference
                .native
                .lines()
                .find_map(|line| line.strip_prefix(prefix))
                .expect("Native must emit a body travel acceleration command")
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native travel commands must retain separate support while applying only admitted machine caps"
            );
        }),
    );
}
