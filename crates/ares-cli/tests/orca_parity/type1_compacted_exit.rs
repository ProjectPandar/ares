//! GCode.cpp::WipeTowerIntegration::append_tcr (1640–1660).

use crate::{
    golden,
    runner::{application, native_stream_case},
};
use std::fs;

const PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/type1-compacted-exit/x1c-two-slots.3mf");
const SHA: &str = "6037aa73237ff2a8ab7eedb7e82ee389d96b82066cd80023125e4d9bb07959de";

fn compacted_exits(gcode: &str) -> Vec<([&str; 3], &str)> {
    gcode
        .split(
            "; WIPE_TOWER_END

",
        )
        .skip(1)
        .filter_map(|block| {
            let mut lines = block.lines();
            let [retract, raise, unretract] = [lines.next()?, lines.next()?, lines.next()?];
            if !(retract.starts_with("G1 E-")
                && raise.starts_with("G1 Z")
                && unretract.starts_with("G1 E")
                && !unretract.starts_with("G1 E-"))
            {
                return None;
            }
            let (_, object) = block.split_once("; printing object ").unwrap();
            let (entry, _) = object.split_once("; FEATURE:").unwrap();
            Some(([retract, raise, unretract], entry))
        })
        .collect()
}

#[test]
fn type1_compacted_tower_exits_without_trailing_wipe() {
    let (reference, _) = native_stream_case::qualify(
        ("type1-compacted-exit", &["GCode.cpp:1640–1660"]),
        "x1c-two-slots",
        PROJECT,
        SHA,
        true,
    );
    let expected = compacted_exits(&reference.native);
    assert!(!expected.is_empty(), "Native must return from compacted Z");
    for (_, entry) in &expected {
        assert!(
            !entry.contains("; WIPE_START"),
            "Native object entry: {entry}"
        );
        assert!(entry.lines().any(|line| line.starts_with("G1 X")));
    }

    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            PROJECT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "CLI slice failed: {:?}", cli.stderr);
    let cli = fs::read(cli_path).unwrap();

    for (producer, output) in [("API", api), ("CLI", cli)] {
        let text = std::str::from_utf8(&output).unwrap();
        assert_eq!(
            compacted_exits(text),
            expected,
            "{producer} compacted tower exit and next object entry at {:?}",
            reference.root
        );
        golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &output)
            .unwrap_or_else(|error| panic!("{producer} at {:?}: {error}", reference.root));
    }
}
