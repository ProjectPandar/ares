//! `GCodeWriter.cpp:1056-1081` / `GCodeWriter.hpp:266-269`: scan restore comment.

#[test]
fn scan_unretract_keeps_literal_source_comment() {
    let project = include_bytes!(
        "../../../../tests/parity/scan-unretract-comment/scan-unretract-comments.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "scan-unretract-comment",
            &[
                "GCode.cpp:4782-4789",
                "GCodeWriter.cpp:1056-1081::unretract",
                "GCodeWriter.hpp:266-269::emit_comment",
            ],
        ),
        "scan-unretract-comments",
        project,
        "f75d406432bc3bcb659f68705ab24faef2e345c1d7687141ef73bb256483a071",
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
            assert_eq!(settings["use_firmware_retraction"], "0");
            assert_eq!(settings["use_relative_e_distances"], "1");
            let scan_enabled = settings["scan_first_layer"] == "1";
            let comments = settings["gcode_comments"] == "1";
            let lines: Vec<_> = reference.native.lines().collect();
            let marker = lines.iter().position(|line| line.starts_with("M976 S1 P1"));
            if scan_enabled {
                let following = &lines[marker.expect("Native must inspect the first layer") + 1..];
                let wait = following
                    .iter()
                    .position(|line| *line == "M400 P100")
                    .expect("Native inspection must include its wait");
                let restore = following[wait + 1..]
                    .iter()
                    .take_while(|line| !line.starts_with("M204"))
                    .find(|line| line.starts_with("G1 E"))
                    .expect("Native must restore E before resetting acceleration");
                let delta: f64 = restore
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('E'))
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(
                    delta > 0.0,
                    "Qualify an ordinary nonzero relative-E restore"
                );
                assert!(
                    !restore
                        .split_ascii_whitespace()
                        .any(|word| word.starts_with(['X', 'Y', 'Z']))
                );
                if comments {
                    assert!(
                        restore.ends_with(" ;  ; unretract"),
                        "Preserve both source comment separators"
                    );
                } else {
                    assert!(
                        !restore.contains(';'),
                        "Native stock restore has comments disabled"
                    );
                }
            } else {
                assert!(
                    marker.is_none(),
                    "Native scan-disabled control must omit inspection"
                );
            }
        }),
    );
}

#[test]
fn stock_scan_unretract_control_omits_comments() {
    let project =
        include_bytes!("../../../../tests/parity/scan-unretract-comment/stock-control.3mf");
    crate::runner::native_stream_case::check(
        (
            "scan-unretract-comment",
            &[
                "GCode.cpp:4782-4789",
                "GCodeWriter.cpp:1056-1081::unretract",
                "GCodeWriter.hpp:266-269::emit_comment",
            ],
        ),
        "stock-control",
        project,
        "748222a3819ea8ad5c3a346bac61f65149dd753bfc12f468b51d8b9e7cf96c0c",
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
            assert_eq!(settings["use_firmware_retraction"], "0");
            assert_eq!(settings["use_relative_e_distances"], "1");
            let scan_enabled = settings["scan_first_layer"] == "1";
            let comments = settings["gcode_comments"] == "1";
            let lines: Vec<_> = reference.native.lines().collect();
            let marker = lines.iter().position(|line| line.starts_with("M976 S1 P1"));
            if scan_enabled {
                let following = &lines[marker.expect("Native must inspect the first layer") + 1..];
                let wait = following
                    .iter()
                    .position(|line| *line == "M400 P100")
                    .expect("Native inspection must include its wait");
                let restore = following[wait + 1..]
                    .iter()
                    .take_while(|line| !line.starts_with("M204"))
                    .find(|line| line.starts_with("G1 E"))
                    .expect("Native must restore E before resetting acceleration");
                let delta: f64 = restore
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('E'))
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(
                    delta > 0.0,
                    "Qualify an ordinary nonzero relative-E restore"
                );
                assert!(
                    !restore
                        .split_ascii_whitespace()
                        .any(|word| word.starts_with(['X', 'Y', 'Z']))
                );
                if comments {
                    assert!(
                        restore.ends_with(" ;  ; unretract"),
                        "Preserve both source comment separators"
                    );
                } else {
                    assert!(
                        !restore.contains(';'),
                        "Native stock restore has comments disabled"
                    );
                }
            } else {
                assert!(
                    marker.is_none(),
                    "Native scan-disabled control must omit inspection"
                );
            }
        }),
    );
}

#[test]
fn scan_disabled_comment_control_has_no_inspection_block() {
    let project = include_bytes!(
        "../../../../tests/parity/scan-unretract-comment/scan-disabled-comments-control.3mf"
    );
    crate::runner::native_stream_case::check(
        (
            "scan-unretract-comment",
            &[
                "GCode.cpp:4782-4789",
                "GCodeWriter.cpp:1056-1081::unretract",
                "GCodeWriter.hpp:266-269::emit_comment",
            ],
        ),
        "scan-disabled-comments-control",
        project,
        "4725806a943799978387ac2f1809e9dbbab5bfa7cbf9224db654cb7ae97bec31",
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
            assert_eq!(settings["use_firmware_retraction"], "0");
            assert_eq!(settings["use_relative_e_distances"], "1");
            let scan_enabled = settings["scan_first_layer"] == "1";
            let comments = settings["gcode_comments"] == "1";
            let lines: Vec<_> = reference.native.lines().collect();
            let marker = lines.iter().position(|line| line.starts_with("M976 S1 P1"));
            if scan_enabled {
                let following = &lines[marker.expect("Native must inspect the first layer") + 1..];
                let wait = following
                    .iter()
                    .position(|line| *line == "M400 P100")
                    .expect("Native inspection must include its wait");
                let restore = following[wait + 1..]
                    .iter()
                    .take_while(|line| !line.starts_with("M204"))
                    .find(|line| line.starts_with("G1 E"))
                    .expect("Native must restore E before resetting acceleration");
                let delta: f64 = restore
                    .split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('E'))
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(
                    delta > 0.0,
                    "Qualify an ordinary nonzero relative-E restore"
                );
                assert!(
                    !restore
                        .split_ascii_whitespace()
                        .any(|word| word.starts_with(['X', 'Y', 'Z']))
                );
                if comments {
                    assert!(
                        restore.ends_with(" ;  ; unretract"),
                        "Preserve both source comment separators"
                    );
                } else {
                    assert!(
                        !restore.contains(';'),
                        "Native stock restore has comments disabled"
                    );
                }
            } else {
                assert!(
                    marker.is_none(),
                    "Native scan-disabled control must omit inspection"
                );
            }
        }),
    );
}
