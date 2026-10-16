//! Public RED for SpiralVase final transition-out E precision.
//! Upstream: GCode.cpp:4596-4612 and SpiralVase.cpp:123-160.

use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

use super::OrcaRunner;

const TWO_ISLANDS: &[u8] =
    include_bytes!("../../../../../tests/parity/arachne-alternate-extra-wall/two-islands-on.3mf");
const SPIRAL_SHA: &str = "60154fac5cadb3cb49a1e3589317434a153856d5967edfa8734b885d8bce7382";
const REGULAR_SHA: &str = "c85752e3d537f6113ef71d8c564532d95a2e33334bc977d6a7d4d16244110dba";

#[test]
fn spiral_transition_out_two_islands_matches_native() {
    compare("spiral", variant(true));
}

#[test]
fn spiral_transition_out_two_islands_regular_control_matches_native() {
    compare("regular", variant(false));
}

fn variant(spiral: bool) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(TWO_ISLANDS)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let name = file.name().to_owned();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if name == "Metadata/project_settings.config" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "\"spiral_mode\": \"0\"",
                    &format!("\"spiral_mode\": \"{}\"", usize::from(spiral)),
                )
                .into_bytes();
        }
        output
            .start_file(name, zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn compare(label: &str, input: Vec<u8>) {
    let expected = if label == "spiral" {
        SPIRAL_SHA
    } else {
        REGULAR_SHA
    };
    assert_eq!(
        crate::runner::cache_identity::raw_digest(&input),
        expected,
        "{label} input manifest"
    );
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("spiral-transition-red-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = root.join("input.3mf");
    fs::write(&input_path, &input).unwrap();
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    crate::runner::stages::run(
        runner.bin(),
        [
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            native_dir.to_str().unwrap().into(),
            input_path.to_str().unwrap().into(),
        ],
        crate::runner::stages::Stage::Slice,
        &root,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &input,
            ares_core::GenerationMetadata::deterministic(2026, 10, 17, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    let api_comparison = api
        .as_ref()
        .map_err(|error| format!("{error:?}"))
        .and_then(|bytes| crate::golden::compare_ordered_bytes_generator_only(&native, bytes));
    let cli_output = root.join("ares.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_output)
        .arg(&input_path)
        .output()
        .unwrap();
    assert!(cli.status.success(), "{label}: {cli:?}");
    let cli_bytes = fs::read(&cli_output).unwrap();
    assert_eq!(
        api.as_ref().ok(),
        Some(&cli_bytes),
        "{label} API/CLI raw bytes; artifacts {root:?}"
    );
    assert_eq!(api_comparison, Ok(()), "{label} API; artifacts {root:?}");
    assert_eq!(
        crate::golden::compare_ordered_bytes_generator_only(&native, &cli_bytes),
        Ok(()),
        "{label} CLI; artifacts {root:?}"
    );
}
