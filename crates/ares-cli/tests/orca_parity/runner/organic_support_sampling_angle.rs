//! Beta TreeSupport3D.cpp:749-756 roof sampling angle and public support output.
use crate::runner::native_stream_case::{check, qualify};

const PROJECT: &[u8] =
    include_bytes!("../../../../../tests/parity/skirt-support-occupied-input/support-overhang.3mf");
const SHA: &str = "58bf8c9a7a28cc19d7ee61c45458ae319dbce6882036cc7a7f6dde06d60c288d";
const UPSTREAM: &[&str] = &["Support/TreeSupport3D.cpp:749-756::generate_support_infill_lines"];
const FIRST_SUPPORT: &str = "G1 X102.572 Y101.777 E.03026";

#[test]
fn organic_sampling_angle_first_support_command_matches_native() {
    let (reference, replay) = qualify(
        ("organic-sampling-angle-first-support", UPSTREAM),
        "support-overhang",
        PROJECT,
        SHA,
        false,
    );
    for native in [&reference, &replay] {
        assert_eq!(first_support_command(&native.native), FIRST_SUPPORT);
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            PROJECT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    assert_eq!(
        first_support_command(std::str::from_utf8(&api).unwrap()),
        first_support_command(&reference.native),
    );
}

#[test]
fn organic_sampling_angle_matches_native_complete_output() {
    check(
        ("organic-sampling-angle-complete", UPSTREAM),
        "support-overhang",
        PROJECT,
        SHA,
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_eq!(first_support_command(&native.native), FIRST_SUPPORT);
            }
        }),
    );
}

fn first_support_command(gcode: &str) -> &str {
    let (_, support) = gcode.split_once(";TYPE:Support\n").unwrap();
    support
        .lines()
        .find(|line| {
            line.starts_with("G1 ")
                && line.contains(" E")
                && (line.contains(" X") || line.contains(" Y"))
        })
        .unwrap()
}
