//! `GCode::_extrude` (GCode.cpp:6516-6531) selects default speeds by
//! role. Only erPerimeter/erExternalPerimeter receive the sloped-path
//! scarf cap; erOverhangPerimeter keeps bridge_speed. For this K1
//! `seam_slope_type=external` fixture the defaults are 30 vs the loop's
//! scarf speed 200. Returning the latter for Overhang wall makes the
//! `variable_speed` predicate (GCode.cpp:6714-6715) spuriously true,
//! replacing raw Line3 lengths (GCode.cpp:6993-7046) with quantized
//! processed lengths (GCode.cpp:7114-7248), changing E.02831/E.03072
//! into E.02832/E.03071 despite identical printed XYZ.

#[test]
fn k1_scarf_overhang_role_speed() {
    crate::runner::native_stream_case::check(
        (
            "scarf-role-speed",
            &[
                "GCode.cpp:_extrude:6516-6531:role-defaults-and-perimeter-only-scarf-cap",
                "GCode.cpp:6714-6715:variable_speed-compares-against-role-default",
                "GCode.cpp:6993-7046:constant-sloped-extrusion-uses-raw-Line3-lengths",
            ],
        ),
        "k1-external",
        include_bytes!("../../../../../tests/parity/scarf-role-speed/k1-external.3mf"),
        "408473e9b538433465e8b4a5d6e9371c5a89dead71e9ee0c8d3e2b432d974449",
        (false, |_, _| {}),
    );
}
