//! GCode.cpp:7604 queries Auto-lift overhangs in current-plate coordinates.

use crate::runner::native_stream_case;

const DIR: &str = "auto-lift-query-frame";
const LABEL: &str = "p1s-bylayer-two-objects";
const SHA: &str = "f22c2132032860447369a3b8d29291230108f20ef1030cb532447cfb90ae31dd";
const PROJECT: &[u8] = include_bytes!("../../../../tests/parity/auto-lift-query-frame/project.3mf");
const UPSTREAM: &[&str] = &[
    "GCode.cpp::needs_retraction:7593-7604;Auto Spiral/Slope:7607-7615,7635-7647",
    "GCodeWriter.hpp::get_xy_offset:98-99",
];

fn assert_native_witness(native: &str) {
    for marker in [
        "; printer_model = Bambu Lab P1S\n",
        "; print_sequence = by layer\n",
        "; z_hop_types = Auto Lift\n",
        "; model label id: 45,56\n",
        "; object ids of layer 1 start: 45,56\n",
        "; printing object overhang.obj id:0 copy 0\n",
        "; printing object cube_with_hole.obj id:0 copy 0\n",
    ] {
        assert!(native.contains(marker), "Native witness missing {marker:?}");
    }
    assert!(native.contains("G17\nG3 Z4 I.058 J-1.216 P1  F30000\n"));
    assert!(native.contains("G0 X125 Y260.0\nG0 Z-1.01\nG0 X131 F211\nG0 X124\nM73 P29 R13\n"));
    assert!(!native.contains("M73 P28 R13\n"));
}

#[test]
fn auto_lift_query_frame_p1s() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        LABEL,
        PROJECT,
        SHA,
        (false, |reference, replay| {
            assert_native_witness(&reference.native);
            assert_native_witness(&replay.native);
        }),
    );
}
