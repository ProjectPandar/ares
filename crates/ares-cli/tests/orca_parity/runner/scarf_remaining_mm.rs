//! `ExtrusionLoopSloped` (ExtrusionEntity.cpp:480-501) compares and
//! subtracts path lengths in mm, converting only the split distance to
//! scaled coordinates. Moving that conversion before subtraction changes
//! the K1 length40 cut point by one coord_t unit. Recursive bisection
//! then prints X109.272 rather than Native X109.273 (line 3956).

#[test]
fn k1_scarf_length40_remaining_mm() {
    crate::runner::native_stream_case::check(
        (
            "scarf-remaining-mm",
            &[
                "GCode.cpp:5945-5951:scarf-length-constructor-input",
                "ExtrusionEntity.cpp:480-501:remaining-and-path-length-stay-in-mm",
                "Polyline.cpp:892-918:split_at_length-interpolation",
            ],
        ),
        "k1-all-length40",
        include_bytes!("../../../../../tests/parity/scarf-remaining-mm/k1-all-length40.3mf"),
        "a8f8ebed0903b58347cc4b423e2754e90008dd9b4295fd0f576d8afc0e4b1856",
        (false, |_, _| {}),
    );
}
