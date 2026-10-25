//! GCode.cpp:5326–5493 wiping two-pass object-label RED; GREEN is not authorized.
use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "mm-flush-into-infill-labels",
    &[
        "GCode.cpp:5482-5493::process_layer object label drop/arm",
        "GCode.cpp:5326-5329::wiping two-pass loop",
    ],
);

#[test]
fn mm_flush_into_infill_labels_target_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "target",
        include_bytes!("../../../../tests/parity/mm-flush-into-infill-labels/project.3mf"),
        "c80f9ebb4465e426e6f74b49d1f68c99dc0556b59a24a0f780e9b37e86181c6a",
        (true, |reference, _replay| {
            // The knob must activate the wiping/normal two-pass
            // (`GCode.cpp:5326-5329`) and both toolchange tools.
            assert!(
                reference.native.contains("; PURGING FINISHED\n"),
                "target must activate the wiping two-pass: {:?}",
                reference.root
            );
            // `GCode.cpp:5482-5493` keeps the Native envelope balanced:
            // every emitted M624 start has exactly one matching M625 stop.
            let starts = reference
                .native
                .matches("; start printing object, unique label id:")
                .count();
            let stops = reference
                .native
                .matches("; stop printing object, unique label id:")
                .count();
            assert_eq!(
                starts, stops,
                "Native envelope balance: {:?}",
                reference.root
            );
        }),
    );
}

#[test]
fn mm_flush_into_infill_labels_base_control_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "control",
        include_bytes!("../../../../tests/parity/mm-flush-into-infill-labels/base.3mf"),
        "8e09d0d146adf8cd9f444c2c343396e772e508e185a69eefa5700733cb8043d0",
        (false, |reference, _replay| {
            // flush_into_infill=0 keeps the single-pass path; this control
            // is byte-green today and must stay so through GREEN.
            assert!(
                !reference.native.contains("; PURGING FINISHED\n"),
                "control must stay single-pass: {:?}",
                reference.root
            );
        }),
    );
}
