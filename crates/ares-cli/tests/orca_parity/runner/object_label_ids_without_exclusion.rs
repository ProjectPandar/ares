//! Beta GCode.cpp:4336-4340,11356-11370 assigns IDs before the exclusion gate.
use crate::runner::native_stream_case::qualify;

#[test]
fn object_label_ids_are_assigned_without_exclusion() {
    let bytes = include_bytes!(
        "../../../../../tests/parity/object-label-ids-without-exclusion/overhang.3mf"
    );
    let (reference, _) = qualify(
        (
            "object-label-ids-without-exclusion",
            &["GCode.cpp:4336-4340", "GCode.cpp:11356-11370"],
        ),
        "overhang",
        bytes,
        "71e2690d381d7698a94ce5c6d53eb2da5359fe00de18029c26c10a8b707a1695",
        false,
    );
    assert!(reference.native.contains("; exclude_object = 0"));
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let expected = labels(&reference.native);
    assert!(expected.iter().any(|line| line.contains(" id:1 copy 0")));
    assert_eq!(labels(std::str::from_utf8(&api).unwrap()), expected);
}

fn labels(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| {
            line.starts_with("; printing object ") || line.starts_with("; stop printing object ")
        })
        .collect()
}
