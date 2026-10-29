//! `GCodeWriter.cpp:38-41`: machine print-acceleration caches use lrint.

#[test]
fn even_half_machine_cap_rounds_to_even() {
    let project = include_bytes!(
        "../../../../tests/parity/machine-acceleration-cache-rounding/even-half-extruding.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "machine-acceleration-cache-rounding",
            &["GCodeWriter.cpp:38-41::apply_print_config"],
        ),
        "even-half-extruding",
        project,
        "110b8b97b5faabd9365e3dae2d1fb9f15339dff4337da95f96337c8ec8952977",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let cap: f64 = settings["machine_max_acceleration_extruding"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let requested: f64 = settings["initial_layer_acceleration"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let expected = requested.round().min(cap.round_ties_even()) as u32;
            let actual: u32 = reference
                .native
                .lines()
                .find_map(|line| line.strip_prefix("M204 S"))
                .expect("Native must emit a body acceleration command")
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native first-layer body acceleration must use the source lrint cache, not raw envelope rounding"
            );
        }),
    );
}

#[test]
fn stock_machine_cap_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "machine-acceleration-cache-rounding",
            &["GCodeWriter.cpp:38-41::apply_print_config"],
        ),
        "stock-control",
        project,
        "b1a3ee564b1e78d1b87f6957f2177f6cd528906ce4790218a0dca410b4191bb8",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let cap: f64 = settings["machine_max_acceleration_extruding"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let requested: f64 = settings["initial_layer_acceleration"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let expected = requested.round().min(cap.round_ties_even()) as u32;
            let actual: u32 = reference
                .native
                .lines()
                .find_map(|line| line.strip_prefix("M204 S"))
                .expect("Native must emit a body acceleration command")
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native first-layer body acceleration must use the source lrint cache, not raw envelope rounding"
            );
        }),
    );
}

#[test]
fn odd_half_machine_cap_control_stays_byte_equal() {
    let project = include_bytes!(
        "../../../../tests/parity/machine-acceleration-cache-rounding/odd-half-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "machine-acceleration-cache-rounding",
            &["GCodeWriter.cpp:38-41::apply_print_config"],
        ),
        "odd-half-control",
        project,
        "0eca982752c861b52a980dd46dd0fc4514748780705fe30a99cc1c9f3d9c595b",
        (false, |reference, _| {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
            let settings: serde_json::Value = serde_json::from_reader(
                archive.by_name("Metadata/project_settings.config").unwrap(),
            )
            .unwrap();
            let cap: f64 = settings["machine_max_acceleration_extruding"][0]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let requested: f64 = settings["initial_layer_acceleration"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let expected = requested.round().min(cap.round_ties_even()) as u32;
            let actual: u32 = reference
                .native
                .lines()
                .find_map(|line| line.strip_prefix("M204 S"))
                .expect("Native must emit a body acceleration command")
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                actual, expected,
                "Native first-layer body acceleration must use the source lrint cache, not raw envelope rounding"
            );
        }),
    );
}
