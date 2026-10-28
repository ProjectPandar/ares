//! `OrcaSlicer.cpp:6027-6034`: grow short maps and reset only logical slots.

use crate::runner::native_stream_case::check;

#[test]
fn orca_parity_filament_map_longer_than_consumables_matches_complete_output() {
    check(
        ("filament-map-length", &["OrcaSlicer.cpp:6027-6034"]),
        "x1c-cube-too-long",
        include_bytes!("../../../../../tests/parity/filament-map-length/x1c-cube-too-long.3mf"),
        "3de8de84dee7bbbdb45530a0c247e79896f43ed253f69bb6a5020cc46375ca13",
        (false, |_, _| {}),
    );
}
