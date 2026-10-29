//! Native Clipper Orientation classifies generated zero-area loops as CCW.

#[test]
fn fuzzy_external_zero_area_orientation_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "zero-area-orientation",
            &[
                "GCode.cpp:6003,6011",
                "ExtrusionEntity.cpp:162-171",
                "Polygon.cpp:71-74",
                "deps_src/clipper/clipper.hpp:200",
                "deps_src/clipper/clipper.cpp:167-179",
            ],
        ),
        "fuzzy-external-target",
        include_bytes!("../../../../tests/parity/zero-area-orientation/fuzzy-external-target.3mf"),
        "b71a3657d0fe805993e39c680c10310583446f67e94b7a671fc9ec4332900659",
        (false, |_, _| {}),
    );
}

#[test]
fn fuzzy_none_sole_key_control() {
    crate::runner::native_stream_case::check(
        (
            "zero-area-orientation",
            &[
                "GCode.cpp:6003,6011",
                "ExtrusionEntity.cpp:162-171",
                "Polygon.cpp:71-74",
                "deps_src/clipper/clipper.hpp:200",
                "deps_src/clipper/clipper.cpp:167-179",
            ],
        ),
        "fuzzy-none-control",
        include_bytes!("../../../../tests/parity/zero-area-orientation/fuzzy-none-control.3mf"),
        "06693138931240d74774230ef570d19d65a02afe9ded7d0da97648693a67822a",
        (false, |_, _| {}),
    );
}
