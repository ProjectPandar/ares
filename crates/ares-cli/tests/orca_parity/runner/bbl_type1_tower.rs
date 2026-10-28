//! Print.hpp:1086 and Print.cpp:3444-3541: stock BBL Type1 tower parity.
//! The width60 witness pins the `prime_tower_skip_points` waypoint travels
//! (`GCode.cpp:1002-1046` → `generate_path_to_wipe_tower`, `GCode.cpp:594-710`):
//! the polyline `Point`s flow straight into `travel_to(p)` / `point_to_gcode`
//! (`GCode.cpp:8124-8127`), and the post-toolchange slope lift
//! (`GCodeWriter.cpp:685-770`) reads the gridded writer position — at
//! `prime_tower_width` 60 the slope top lands on the 3-decimal rounding
//! boundary (Y221.158 vs Y221.159, 13 occurrences).

const UPSTREAM: &[&str] = &[
    "Print.hpp:1086",
    "Print.cpp:3290-3301,3390-3541",
    "GCode/WipeTower.cpp:2449-2510,3516-3806,3864",
    "GCode.cpp:712-1117,2926-2939",
];

#[test]
fn bbl_type1_tower_enabled() {
    crate::runner::native_stream_case::check(
        ("bbl-type1-tower", UPSTREAM),
        "enabled",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/enabled.3mf"),
        "ab00b9040c3ccc1c10490a220880b87108aed7edd3a19926b8e7967f1287e506",
        (false, |_, _| {}),
    );
}

#[test]
fn bbl_type1_tower_width60_skip_points_slope_boundary() {
    crate::runner::native_stream_case::check(
        (
            "bbl-type1-tower",
            &[
                "GCode.cpp:594-710,1002-1046",
                "GCodeWriter.cpp:685-770",
                "GCode.cpp:8124-8127",
            ],
        ),
        "width60",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/width60.3mf"),
        "8bea13f0c36e384615a1c9dea4960ba62a5395be43d8c15e15acca21cee85563",
        (true, |_, _| {}),
    );
}

#[test]
fn bbl_type1_tower_rib3_segment_unscale() {
    crate::runner::native_stream_case::check(
        (
            "bbl-type1-tower",
            &["GCode/WipeTower.cpp:883-894,1102-1153", "Point.hpp:688-705"],
        ),
        "rib3",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/rib3.3mf"),
        "986e7ea38d1f60ff26e086c0ae8cdb4ba68336ba5d442689276deba603768f5d",
        (false, |_, _| {}),
    );
}

#[test]
fn bbl_type1_tower_disabled_control() {
    crate::runner::native_stream_case::check(
        ("bbl-type1-tower", UPSTREAM),
        "disabled",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/disabled.3mf"),
        "e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28",
        (false, |_, _| {}),
    );
}
