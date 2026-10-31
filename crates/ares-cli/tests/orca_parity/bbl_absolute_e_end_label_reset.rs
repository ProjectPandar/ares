//! `GCodeWriter.cpp:1181-1192`: flushing an end label resets absolute E,
//! including when E is already zero (`reset_e(true)`, :480-502).

use crate::runner::native_stream_case;

#[test]
fn bbl_absolute_e_end_label_reset() {
    native_stream_case::check(
        (
            "bbl-absolute-e-end-label-reset",
            &["GCodeWriter.cpp:1181-1198,480-502", "GCode.cpp:5699-5705"],
        ),
        "a1-byobject-absolute-e",
        include_bytes!(
            "../../../../tests/parity/bbl-absolute-e-end-label-reset/a1-byobject-absolute-e.3mf"
        ),
        "db715a8ceefbbb43e2a8b12a42b2197a5d87592022c857a4ded68f7bd646554b",
        (false, |reference, _| {
            assert!(
                reference
                    .native
                    .contains("M82 ; use absolute distances for extrusion\n")
            );
            assert!(reference.native.contains("; total layer number: 81\n"));
            assert!(reference.native.contains("M625\nG92 E0\nG1 Z.4 F42000\n"));
        }),
    );
}
