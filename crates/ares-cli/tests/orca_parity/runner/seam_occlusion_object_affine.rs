//! `SeamPlacer.cpp:630-671` occlusion object affine uses
//! `its_transform<double>` (`admesh/stl.h:362-369`), then stores float vertices.
use crate::runner::native_stream_case::check;

#[test]
fn aligned_back_occlusion_object_affine_matches_native_complete_output() {
    check(
        (
            "seam-occlusion-object-affine",
            &[
                "GCode/SeamPlacer.cpp:630-671:compute_global_occlusion",
                "Print.hpp:325-328:trafo_centered",
                "Point.hpp:85:Transform3d-double",
                "admesh/stl.h:362-369:its_transform-double-then-float",
            ],
        ),
        "x1c-thin-0",
        include_bytes!("../../../../../tests/parity/seam-occlusion-object-affine/x1c-thin-0.3mf"),
        "eb7bdc6e74029dfa25da954c231142f59eb37da648943a425516f1eb54dd894e",
        (false, |_, _| {}),
    );
}
