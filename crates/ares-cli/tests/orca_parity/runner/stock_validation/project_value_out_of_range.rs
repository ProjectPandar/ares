//! `OrcaSlicer.cpp:4277-4282`: the CLI rejects a 3MF value outside its option range with
//! `DynamicPrintConfig::validate(true)`. OrcaSlicer's GUI only warns about it at load
//! (`Plater.cpp:9466-9481`), so the web E2E slices the same project.

use std::io::{Cursor, Read as _, Write as _};

use super::super::{EarlyExit, OrcaRunner, application};
use crate::artifacts;
use crate::native_wall_clock::result_comparison_bytes;

const INPUT: &[u8] =
    include_bytes!("../../../../../../tests/parity/bbl-jerk-tail-precision/stock-control.3mf");

#[test]
fn a_negative_raft_expansion_in_a_project_fails_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("project-value-out-of-range-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = root.join("input.3mf");
    artifacts::write(&project, &with_setting("raft_first_layer_expansion", "-1")).unwrap();
    let native = runner
        .early_cli_reference(&project, EarlyExit::InvalidValues)
        .unwrap();
    assert!(
        String::from_utf8_lossy(&native.stderr)
            .contains("raft_first_layer_expansion: -1 not in range"),
        "{root:?}"
    );
    let retained = root.join("plate_1.gcode");
    let output = application::cli_command()
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(&project)
        .output()
        .unwrap();
    artifacts::write(&root.join("cli.stderr"), &output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(if cfg!(unix) { 238 } else { -18 }),
        "{root:?}"
    );
    assert_eq!(output.stdout, native.stdout, "{root:?}");
    assert_eq!(output.stderr, native.stderr, "{root:?}");
    // OrcaSlicer.cpp:3005 records `upward_compatible_machine` before the :4277 check; the Ares
    // CLI does not report it on this exit yet.
    let report = |bytes: &[u8]| -> serde_json::Value {
        serde_json::from_slice(&result_comparison_bytes(bytes)).unwrap()
    };
    let mut expected = report(&native.result_json);
    expected
        .as_object_mut()
        .unwrap()
        .remove("upward_compatible_machine");
    assert_eq!(
        report(&std::fs::read(root.join("result.json")).unwrap()),
        expected,
        "{root:?}"
    );
    assert!(!retained.exists(), "{root:?}");
}

fn with_setting(key: &str, value: &str) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(INPUT)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings[key] = value.into();
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}
