//! Beta `PerimeterGenerator.cpp:68,1507,1699-1717,303`: Classic thin-hole reach.

use crate::runner::native_stream_case;

#[test]
fn one_wall_top_hole_matches_native_complete_output() {
    native_stream_case::check(
        (
            "classic-thin-hole-reach",
            &[
                "PerimeterGenerator.cpp::PerimeterGeneratorLoop::is_thin_wall_hole:68",
                "PerimeterGenerator.cpp::gap_fill_min_width:1507;first-shell reach:1699-1717",
                "PerimeterGenerator.cpp::traverse_loops:303",
            ],
        ),
        "sequential",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-arc-enabled/sequential.3mf"
        )),
        "baa6c8d352d5e62c400cc450d50de643ce868fcc619a71352748f89aabe7aab1",
        (false, |reference, _| {
            for marker in [
                "; wall_generator = classic\n",
                "; wall_direction = ccw\n",
                "; wall_loops = 2\n",
                "; only_one_wall_top = 1\n",
                "; wall_sequence = inner wall/outer wall\n",
                "; printing object cube_with_hole.obj id:1 copy 0\n",
                ";TYPE:Outer wall\n",
            ] {
                assert!(reference.native.contains(marker));
            }
        }),
    );
}
