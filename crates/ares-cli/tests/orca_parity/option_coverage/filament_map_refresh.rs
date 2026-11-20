//! CLI:6974-6981 retains an overlong map; ToolOrdering writes its recommendation.

use crate::runner::native_stream_case::check;
use std::io::{Cursor, Read};

#[test]
fn filament_map_refresh_replaces_overlong_applied_map_with_recommendation() {
    let project =
        include_bytes!("../../../../../tests/parity/filament-map-length/x1c-cube-too-long.3mf");
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut model_settings = String::new();
    archive
        .by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut model_settings)
        .unwrap();
    let applied_map = model_settings
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("<metadata key=\"filament_maps\" value=\"")?
                .strip_suffix("\"/>")
        })
        .unwrap();
    assert_eq!(
        applied_map.split_whitespace().collect::<Vec<_>>(),
        ["1", "1"]
    );
    check(
        (
            "filament-map-refresh",
            &[
                "OrcaSlicer.cpp:3851-3862,3932,6974-6981",
                "GCode/ToolOrdering.cpp:1752-1810,3063-3089",
                "Print.cpp:4661-4755",
            ],
        ),
        "x1c-overlong-applied-map",
        project,
        "3de8de84dee7bbbdb45530a0c247e79896f43ed253f69bb6a5020cc46375ca13",
        (false, |reference, replay| {
            for case in [reference, replay] {
                let map = case
                    .native
                    .lines()
                    .find_map(|line| line.strip_prefix("; filament_map = "))
                    .unwrap();
                assert_eq!(map.split(',').count(), 1);
                assert_eq!(map, "1");
            }
        }),
    );
}
