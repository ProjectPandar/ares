//! Beta SupportParameters.hpp:308–329 normal interface selector, not raft parity.
use crate::{
    golden,
    runner::native_stream_case::{check, qualify},
};
use serde_json::json;
use std::fs;

const PROJECT: &[u8] =
    include_bytes!("../../../../../tests/parity/skirt-support-occupied-input/support-overhang.3mf");
const SHA: &str = "58bf8c9a7a28cc19d7ee61c45458ae319dbce6882036cc7a7f6dde06d60c288d";
const NATIVE_FIRST_MOVE: &str = "G1 X104.101 Y104.369 E.31353";
const UPSTREAM: &[&str] = &[
    "Support/SupportParameters.hpp:308-329::support_interface_angle",
    "Support/SupportCommon.cpp:1729,1852-1880::generate_support_toolpaths",
    "Fill/FillRectilinear.cpp:3490-3501::fill_surface",
];

#[test]
fn support_interface_angle_first_move_matches_native() {
    let (reference, replay) = qualify(
        ("support-interface-angle-first-move", UPSTREAM),
        "support-overhang",
        PROJECT,
        SHA,
        false,
    );
    for native in [&reference, &replay] {
        assert_eq!(first_interface_move(&native.native), NATIVE_FIRST_MOVE);
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            PROJECT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let path = reference.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(path).unwrap();
    let api_text = std::str::from_utf8(&api).unwrap();
    let cli_text = std::str::from_utf8(&cli_bytes).unwrap();
    fs::write(
        reference.root.join("focused-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": SHA,
            "native_first_interface_move": first_interface_move(&reference.native),
            "api_first_interface_move": first_interface_move(api_text),
            "cli_first_interface_move": first_interface_move(cli_text),
            "cli_code": cli.status.code(),
            "api_cli_complete_bytes_equal": api == cli_bytes,
            "complete_api_comparison": golden::compare_ordered_bytes_generator_only(
                reference.native.as_bytes(), &api,
            ),
            "complete_cli_comparison": golden::compare_ordered_bytes_generator_only(
                reference.native.as_bytes(), &cli_bytes,
            ),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "CLI at {:?}", reference.root);
    assert!(api == cli_bytes, "complete API/CLI bytes differ");
    assert_eq!(
        first_interface_move(api_text),
        first_interface_move(&reference.native),
        "public API first Support interface deposition",
    );
    assert_eq!(
        first_interface_move(cli_text),
        first_interface_move(&reference.native),
        "public CLI first Support interface deposition",
    );
}

#[test]
fn support_interface_angle_matches_native_complete_output() {
    check(
        ("support-interface-angle-complete", UPSTREAM),
        "support-overhang",
        PROJECT,
        SHA,
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_eq!(first_interface_move(&native.native), NATIVE_FIRST_MOVE);
            }
        }),
    );
}

fn first_interface_move(gcode: &str) -> &str {
    gcode
        .split_once(";TYPE:Support interface\n")
        .unwrap()
        .1
        .lines()
        .take_while(|line| !line.starts_with(";TYPE:"))
        .find(|line| line.starts_with("G1 X") && line.contains(" E"))
        .unwrap()
}
