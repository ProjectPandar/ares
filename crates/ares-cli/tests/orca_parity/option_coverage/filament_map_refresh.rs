//! `Print.cpp:3168-3175` assigns the recommended map without CLI-entry growth.

use crate::runner::native_stream_case::check;

#[test]
fn filament_map_refresh_preserves_recommended_length_complete_output() {
    check(
        (
            "filament-map-refresh",
            &[
                "OrcaSlicer.cpp:6027-6034",
                "ToolOrdering.cpp:1114-1151,1291-1303",
                "Print.cpp:3168-3175",
                "GCode.cpp:5591-5643",
            ],
        ),
        "ender3-two-diameters-one-colour",
        include_bytes!(
            "../../../../../tests/parity/filament-map-refresh/ender3-two-diameters-one-colour.3mf"
        ),
        "dcd367d3ec44e01a4ad6d3ed13c57c527e236b8f3d0a2f9590a0e079b3d53dea",
        (false, |reference, replay| {
            for case in [reference, replay] {
                let diameter = case
                    .native
                    .lines()
                    .find_map(|line| line.strip_prefix("; filament_diameter = "))
                    .unwrap();
                let colour = case
                    .native
                    .lines()
                    .find_map(|line| line.strip_prefix("; filament_colour = "))
                    .unwrap();
                let map = case
                    .native
                    .lines()
                    .find_map(|line| line.strip_prefix("; filament_map = "))
                    .unwrap();
                assert_eq!(diameter.split(',').count(), 2);
                assert_eq!(colour.split(';').count(), 1);
                assert_eq!(map, "1");
            }
        }),
    );
}
