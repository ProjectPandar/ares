use std::io::{Cursor, Read, Write};

use serde_json::{Value, json};

const SUPPORT: &[u8] = include_bytes!(
    "../../../../tests/parity/support-interface-top-layers-base-connect/control.3mf"
);
const ENDER: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");
const UPSTREAM: (&str, &[&str]) = (
    "spiral-inset",
    &[
        "PrintConfig.cpp:2549–2575,2627–2633,7570–7589",
        "SupportParameters.hpp:151–161",
        "FillSpiralInset.cpp:310–496",
    ],
);

#[test]
fn support_interface_keeps_spiral_inset() {
    compare(SUPPORT, "support_interface_pattern", "Support interface");
}

#[test]
fn top_surface_keeps_spiral_inset() {
    compare(ENDER, "top_surface_pattern", "Top surface");
}

#[test]
fn bottom_surface_keeps_spiral_inset() {
    compare(ENDER, "bottom_surface_pattern", "Bottom surface");
}

fn compare(fixture: &[u8], key: &str, role: &str) {
    let project = with_spiral_inset(fixture, key);
    let config_line = format!("; {key} = spiralinset");
    let role_line = format!(";TYPE:{role}");
    crate::runner::native_stream_case::check(
        UPSTREAM,
        key,
        &project,
        &crate::runner::native_api_cli::sha(&project),
        (false, |reference, _| {
            for line in [&config_line, &role_line] {
                assert!(
                    reference.native.lines().any(|actual| actual == line),
                    "missing {line:?} in Native output at {:?}",
                    reference.root,
                );
            }
        }),
    );
}

fn with_spiral_inset(fixture: &[u8], key: &str) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(fixture)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut changed = false;
    for index in 0..input.len() {
        let mut member = input.by_index(index).unwrap();
        let options =
            zip::write::SimpleFileOptions::default().compression_method(member.compression());
        if member.is_dir() {
            output.add_directory(member.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        if member.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            assert_ne!(settings[key], "spiralinset");
            settings[key] = json!("spiralinset");
            bytes = serde_json::to_vec(&settings).unwrap();
            changed = true;
        }
        output.start_file(member.name(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    assert!(changed);
    output.finish().unwrap().into_inner()
}
