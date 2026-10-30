//! `GCodeProcessor.cpp:1829,5444-5466`: `M1020 S<id>` bills the
//! filament-change time when the change script emits no `T` selection
//! (`process_filament_change` `:5512-5601`). The towerless host keeps the
//! whole-stream compare free of the Type1 tower return travel.

const UPSTREAM: &[&str] = &[
    "GCodeProcessor.cpp:1829",
    "GCodeProcessor.cpp:5444-5466,5470-5499,5512-5601",
];

#[test]
fn processor_m1020_bills_filament_change_without_script_t() {
    crate::runner::native_stream_case::check(
        ("processor-m1020", UPSTREAM),
        "towerless-change",
        include_bytes!("../../../../../tests/parity/processor-m1020/towerless-change.3mf"),
        "80b5e4edce475ccd9555c9dff16feef04f62ba47ef61dbaa82fb10066851d29d",
        (false, |_, _| {}),
    );
}
