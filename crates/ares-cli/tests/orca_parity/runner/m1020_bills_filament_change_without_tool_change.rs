//! `GCodeProcessor.cpp:1829,5444-5466`: `M1020 S<id>` bills the
//! filament-change time when the change script emits no `T` selection
//! (`process_filament_change` `:5512-5601`). The towerless host keeps the
//! whole-stream compare free of the Type1 tower return travel.

const UPSTREAM: &[&str] = &[
    "GCodeProcessor.cpp:1829",
    "GCodeProcessor.cpp:5444-5466,5470-5499,5512-5601",
];

#[test]
fn m1020_bills_filament_change_without_tool_change_bills_filament_change_without_script_t() {
    crate::runner::native_stream_case::check(
        ("m1020-filament-change", UPSTREAM),
        "towerless-change",
        include_bytes!("../../../../../tests/parity/m1020-filament-change/towerless-change.3mf"),
        "8a2a9b965621cff0a3a274bda6db68cf97a02f460c8ea9fe375a14c1dba099a7",
        (false, |_, _| {}),
    );
}
