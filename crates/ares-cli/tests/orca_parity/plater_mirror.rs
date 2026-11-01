//! The object menu's "Mirror: Along X/Y/Z Axis" (`docs/architecture/web-plater-mirror.md`):
//! `GLCanvas3D::mirror_selection` (`GLCanvas3D.cpp:3152-3162` at v2.5.0-beta `345efec64`), a
//! relative `Selection::scale_and_translate` by -1 on the axis about the dragging centre
//! (`Selection.cpp:1627-1719`, `mirror` `:1708-1712`) with the instance sync (`:3045-3095`), then
//! `GLCanvas3D::do_mirror` (`:6010-6092`). Native slices the project the GUI exports.

use ares_core::PlaterSession;

use super::rotation_pivot::prism_project;
use super::{assert_oracle_like_native, assert_plate_like_native, assert_transform, load};

/// One 10 mm cube object with two instances, at (90, 110, 5) and (130, 110, 5).
const TWO_INSTANCES: &str = "tests/parity/plater-scale/two-instances.3mf";

/// The prism of `plater_rotation_pivot.rs` with its (0, 0, 0) corner at (100, 100, 0); it loads with
/// its box centre (110, 108, 5) as its offset.
fn prism() -> PlaterSession {
    PlaterSession::load(prism_project("1 0 0 0 1 0 0 0 1 100 100 0")).unwrap()
}

#[test]
fn plater_mirror_along_x_flips_the_prism_about_its_centre_like_native() {
    // The dragging centre is the box centre, the offset, so the offset stays.
    let mut session = prism();
    session.mirror(0, 0, 0).unwrap();
    assert_transform(
        &session,
        [
            -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 110.0, 108.0, 5.0, 1.0,
        ],
    );
    // Native slices the prism's own mesh frame: the box centre (10, 8, 5) taken back off,
    // (110, 108, 5) - diag(-1, 1, 1) (10, 8, 5) = (120, 100, 0).
    let oracle = prism_project("-1 0 0 0 1 0 0 0 1 120 100 0");
    assert_oracle_like_native("mirror-x-prism", &session, oracle, 1);
}

#[test]
fn plater_mirror_along_z_turns_the_prism_upside_down_without_a_drop_like_native() {
    // Flipped about z 5, the prism still spans z 0 to 10, so do_mirror does not move it.
    let mut session = prism();
    session.mirror(0, 0, 2).unwrap();
    assert_transform(
        &session,
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 110.0, 108.0, 5.0, 1.0,
        ],
    );
    // (110, 108, 5) - diag(1, 1, -1) (10, 8, 5) = (100, 100, 10).
    let oracle = prism_project("1 0 0 0 1 0 0 0 -1 100 100 10");
    assert_oracle_like_native("mirror-z-prism", &session, oracle, 1);
}

#[test]
fn plater_mirroring_one_instance_mirrors_its_sibling_like_native() {
    // `synchronize_unselected_instances(GENERAL)`: the sibling takes `(old_j · old_i⁻¹) · curr_i`,
    // diag(1, -1, 1), keeping its XY offset.
    let mut session = load(TWO_INSTANCES);
    session.mirror(0, 0, 1).unwrap();
    for (instance, x) in [(0, 90.0), (1, 130.0)] {
        let actual = session
            .instance_transform(0, instance)
            .unwrap()
            .to_column_major();
        let expected = [
            1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, 110.0, 5.0, 1.0,
        ];
        for (actual, expected) in actual.iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-9,
                "instance {instance}: {actual} != {expected}"
            );
        }
    }
    assert_plate_like_native("mirror-y-siblings", &session, 1);
}
