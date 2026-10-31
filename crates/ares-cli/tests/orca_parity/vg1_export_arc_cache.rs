//! ExportLines::update GCodeProcessor.cpp:771-810 / caller:1458-1471.

#[test]
fn vg1_export_arc_cache_without_virtual_control() {
    crate::runner::native_stream_case::check(
        (
            "vg1-export-arc-cache-no-virtual-control",
            &["GCodeProcessor.cpp:771-810", "GCodeProcessor.cpp:1458-1471"],
        ),
        "control",
        include_bytes!(
            "../../../../tests/parity/vg1-export-arc-cache-no-virtual-control/control.3mf"
        ),
        "61d36dff5dfb995fb3cd1f2bcb3ece0d9e79c5476137b05ddd495d4b901368fb",
        (true, |_, _| {}),
    );
}

#[test]
fn vg1_export_arc_cache_real_macro_target() {
    crate::runner::native_stream_case::check(
        (
            "vg1-export-arc-cache-no-virtual-control",
            &["GCodeProcessor.cpp:771-810", "GCodeProcessor.cpp:1458-1471"],
        ),
        "target",
        include_bytes!(
            "../../../../tests/parity/vg1-export-arc-cache-no-virtual-control/target.3mf"
        ),
        "f6a1b0ab02e4be9f4d3c20e6dcf09d55b0ba37f4b47e3989053de03adefac169",
        (true, |_, _| {}),
    );
}
