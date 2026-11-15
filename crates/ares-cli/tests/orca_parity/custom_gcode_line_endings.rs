//! Raw custom-script line endings survive the processor, not just thumbnail lines.

use std::io::{Cursor, Read as _, Write as _};

use serde_json::Value;

fn assert_custom_line_endings(label: &str, ending: &str) {
    let fixture = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");
    let marker = format!(";LINE_ENDING_FIRST{ending};LINE_ENDING_SECOND{ending}");
    let mut input = zip::ZipArchive::new(Cursor::new(fixture)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut entry = input.by_index(index).unwrap();
        let options = zip::write::SimpleFileOptions::default();
        if entry.is_dir() {
            output.add_directory(entry.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            let start = settings["machine_start_gcode"].as_str().unwrap();
            settings["machine_start_gcode"] = Value::String(format!("{start}\n{marker}"));
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output.start_file(entry.name(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    let project = output.finish().unwrap().into_inner();
    crate::runner::native_stream_case::check(
        (
            "custom-gcode-line-endings",
            &[
                "Orca v2.5.0 GCodeProcessor.cpp:1794-1803 raw CR/LF reader",
                "GCodeProcessor.cpp:1846,1852,1869 ordinary append_line",
                "GCodeProcessor.cpp:1027-1040,1105-1133 ExportLines",
                "GCodeProcessor.cpp:867-881 GCodeFileWriter::append/flush",
            ],
        ),
        label,
        &project,
        &crate::runner::native_api_cli::sha(&project),
        (false, |reference, replay| {
            for case in [reference, replay] {
                assert!(
                    case.native.contains(&marker),
                    "Native changed custom-script line endings: {:?}",
                    case.root
                );
            }
        }),
    );
}

#[test]
fn machine_start_crlf_matches_complete_native_stream() {
    assert_custom_line_endings("machine-start-crlf", "\r\n");
}

#[test]
fn machine_start_lf_matches_complete_native_stream() {
    assert_custom_line_endings("machine-start-lf", "\n");
}
