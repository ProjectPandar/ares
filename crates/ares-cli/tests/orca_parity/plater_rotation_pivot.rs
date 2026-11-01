//! `Selection::rotate` turns an instance about the centre of the smallest sphere enclosing its
//! hull (`Selection.cpp:1177-1210, 2516-2531`). Loading centres a volume on its bounding box and
//! moves that into the instance offset, so the pivot differs from the offset only when the
//! sphere centre differs from the box centre. A prism on the acute triangle (0, 0), (20, 0),
//! (10, 16), 10 high, has its box centre at (10, 8, 5) but its sphere centre at the triangle's
//! circumcentre at mid-height, (10, 4.875, 5): 10² + 4.875² = 11.125² puts all six vertices on it.

use std::f64::consts::FRAC_PI_2;

use ares_core::PlaterSession;

use super::{SINGLE, assert_oracle_like_native, assert_transform, rewrite_archive};
use crate::runner::repo_root;

const PRISM: &str = r#"<mesh>
    <vertices>
     <vertex x="0" y="0" z="0"/>
     <vertex x="20" y="0" z="0"/>
     <vertex x="10" y="16" z="0"/>
     <vertex x="0" y="0" z="10"/>
     <vertex x="20" y="0" z="10"/>
     <vertex x="10" y="16" z="10"/>
    </vertices>
    <triangles>
     <triangle v1="0" v2="2" v3="1"/>
     <triangle v1="3" v2="4" v3="5"/>
     <triangle v1="0" v2="1" v3="4"/>
     <triangle v1="0" v2="4" v3="3"/>
     <triangle v1="2" v2="0" v3="3"/>
     <triangle v1="2" v2="3" v3="5"/>
     <triangle v1="1" v2="2" v3="5"/>
     <triangle v1="1" v2="5" v3="4"/>
    </triangles>
   </mesh>"#;

/// The Ender-3 project with its cube replaced by the prism and the build item at `item`.
pub(super) fn prism_project(item: &str) -> Vec<u8> {
    let input = std::fs::read(repo_root().join(SINGLE)).unwrap();
    rewrite_archive(&input, |name, text| match name {
        "3D/Objects/cube10.stl_1.model" => {
            let start = text.find("<mesh>").unwrap();
            let end = text.find("</mesh>").unwrap() + "</mesh>".len();
            Some(format!("{}{PRISM}{}", &text[..start], &text[end..]))
        }
        "3D/3dmodel.model" => Some(text.replace(
            "transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"",
            &format!("transform=\"{item}\""),
        )),
        _ => None,
    })
}

#[test]
fn plater_rotation_turns_an_asymmetric_prism_about_its_bounding_sphere_centre() {
    // The (0, 0, 0) corner at (100, 100, 0).
    let mut session = PlaterSession::load(prism_project("1 0 0 0 1 0 0 0 1 100 100 0")).unwrap();
    // Loaded with its box centre as the offset: (110, 108, 5).
    assert_transform(
        &session,
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 110.0, 108.0, 5.0, 1.0,
        ],
    );
    // About Z through the sphere centre (110, 104.875, 5): the offset, 3.125 above it in Y,
    // swings to 3.125 left of it; the prism stays on the bed.
    session
        .rotate_instance(0, 0, [0.0, 0.0, FRAC_PI_2])
        .unwrap();
    assert_transform(
        &session,
        [
            0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 106.875, 104.875, 5.0, 1.0,
        ],
    );
    // About Y through the same centre: the offset, 3.125 left of it, swings to 3.125 above it, and
    // the rectangular side y = 0 lands face down 0.125 above the bed; do_rotate drops it there.
    session
        .rotate_instance(0, 0, [0.0, FRAC_PI_2, 0.0])
        .unwrap();
    assert_transform(
        &session,
        [
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 110.0, 104.875, 8.0, 1.0,
        ],
    );
    // Native slices the same world geometry from the prism's own mesh frame: the item is that
    // matrix with the box centre (10, 8, 5) taken back off, (110, 104.875, 8) - R (10, 8, 5) =
    // (105, 94.875, 0).
    let rotated = prism_project("0 1 0 0 0 1 1 0 0 105 94.875 0");
    assert_oracle_like_native("asymmetric-prism-rotation", &session, rotated, 1);
}
