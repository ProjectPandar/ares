//! `AvoidCrossingPerimeters.cpp:1216-1229` merge-points `init_boundary`
//! pads the boundary/grid bbox by `bbox.radius()`; `:1262-1268` rebuild the
//! external boundary when an endpoint leaves that bbox; `:1237-1240` route
//! external travels in the `gcodegen.origin()` frame (`scaled_origin`),
//! not the emission offset frame. With `reduce_crossing_wall=1`, detour 0,
//! by-layer X1C two-object stock, the post-wipe object-switch travel must
//! route around the cube hole (4 travel moves), matching Native exactly.

#[test]
fn x1c_reduce_detour0_object_switch_travel() {
    crate::runner::native_stream_case::check(
        (
            "avoid-crossing-external-frame",
            &[
                "AvoidCrossingPerimeters.cpp:1216-1229:init_boundary-merge-points-bbox-radius-pad",
                "AvoidCrossingPerimeters.cpp:1262-1268:external-bbox-contains-rebuild-gate",
                "AvoidCrossingPerimeters.cpp:1237-1240:scaled-origin-planning-frame",
            ],
        ),
        "x1c-detour0",
        include_bytes!("../../../../../tests/parity/avoid-crossing-external-frame/x1c-detour0.3mf"),
        "f44f09c6b26391787d5877cfafd094bb50cb452d727298d46cd6ab5d45ffa04e",
        (false, |_, _| {}),
    );
}
