//! The "Scale ratios" and "Size" fields (`docs/architecture/web-plater-scale.md`):
//! `GizmoObjectManipulation::change_scale_value` / `change_size_value` and `do_scale`
//! (`GizmoObjectManipulation.cpp:408-481` at v2.5.0-beta `345efec64`), `Selection::scale` with the
//! instance sync (`Selection.cpp:1627-1719, 3045`) and `GLCanvas3D::do_scale`'s drop to the bed
//! (`GLCanvas3D.cpp:5949-5962`). Native slices the project the GUI exports for the session.

use ares_core::{PlaterSession, ScaleEdit};

use super::{SINGLE, assert_plate_like_native, load};

#[path = "plater_scale_to_fit.rs"]
mod scale_to_fit;

/// One 10 mm cube object with two instances, at (90, 110, 5) and (130, 110, 5)
/// (`tests/parity/plater-scale/generate.mjs`).
const SIBLINGS: &str = "tests/parity/plater-scale/two-instances.3mf";

/// The instance's column-major matrix, against `expected`.
fn assert_instance_transform(
    session: &PlaterSession,
    (object, instance): (usize, usize),
    expected: [f64; 16],
) {
    let actual = session
        .instance_transform(object, instance)
        .unwrap()
        .to_column_major();
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "object {object} instance {instance}: {actual:?} != {expected:?}"
        );
    }
}

/// A scale on the diagonal and an offset, column-major.
fn scaled_at([sx, sy, sz]: [f64; 3], [x, y, z]: [f64; 3]) -> [f64; 16] {
    [
        sx, 0.0, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 0.0, sz, 0.0, x, y, z, 1.0,
    ]
}

#[test]
fn plater_uniform_scale_drops_the_scaled_cube_to_the_bed_like_native() {
    // The 10 mm cube at (110, 110, 5) scales by 1.2 about its centre (`setup_cache`'s dragging
    // centre) to 12 mm, so its bottom at z -1 drops back to the bed: offset z 6.
    let mut session = load(SINGLE);
    session
        .set_scale(
            0,
            0,
            ScaleEdit {
                axis: 0,
                value: 120.0,
                uniform: true,
            },
        )
        .unwrap();
    assert_instance_transform(&session, (0, 0), scaled_at([1.2; 3], [110.0, 110.0, 6.0]));
    assert_plate_like_native("scale-uniform", &session, 1);
}

#[test]
fn plater_size_field_scales_one_axis_when_not_uniform_like_native() {
    // Size X 12 mm of the 10 mm cube: X grows by 1.2 about the centre, the bottom stays on the bed.
    let mut session = load(SINGLE);
    session
        .set_size(
            0,
            0,
            ScaleEdit {
                axis: 0,
                value: 12.0,
                uniform: false,
            },
        )
        .unwrap();
    assert_instance_transform(
        &session,
        (0, 0),
        scaled_at([1.2, 1.0, 1.0], [110.0, 110.0, 5.0]),
    );
    assert_plate_like_native("scale-size-x", &session, 1);
}

#[test]
fn plater_scaling_one_instance_scales_its_sibling_like_native() {
    // `synchronize_unselected_instances(GENERAL)` gives the sibling the scaled linear part and the
    // scaled instance's height. Both cubes grow to 12 mm about z 5 and drop to z 6.
    let mut session = load(SIBLINGS);
    assert_eq!(session.project().objects()[0].id(), 2);
    session
        .set_scale(
            0,
            0,
            ScaleEdit {
                axis: 0,
                value: 120.0,
                uniform: true,
            },
        )
        .unwrap();
    assert_instance_transform(&session, (0, 0), scaled_at([1.2; 3], [90.0, 110.0, 6.0]));
    assert_instance_transform(&session, (0, 1), scaled_at([1.2; 3], [130.0, 110.0, 6.0]));
    assert_plate_like_native("scale-siblings", &session, 1);
}
