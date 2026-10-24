//! SeamPlacer occlusion volume vertices narrow only after the double affine.

#[test]
fn model_volume_double_affine_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "seam-occlusion-volume-affine",
            &[
                "SeamPlacer.cpp:637-641::compute_global_occlusion",
                "deps_src/admesh/stl.h:362-366::its_transform",
            ],
        ),
        "beam1-target",
        include_bytes!("../../../../tests/parity/occlusion-volume-affine/beam1-target.3mf"),
        "ad35795f29e68c292d644a0c8019dd96fb1dac2736adfc651692c2d3a14173f1",
        (true, |_, _| {}),
    );
}

#[test]
fn disabled_interlocking_sole_key_control() {
    crate::runner::native_stream_case::check(
        (
            "seam-occlusion-volume-affine",
            &[
                "SeamPlacer.cpp:637-641::compute_global_occlusion",
                "deps_src/admesh/stl.h:362-366::its_transform",
            ],
        ),
        "beam0-control",
        include_bytes!("../../../../tests/parity/occlusion-volume-affine/beam0-control.3mf"),
        "171ce0b6e5160bbe697212fd548beaa5228fd91cd741fb456ddcdf596feebabd",
        (true, |_, _| {}),
    );
}
