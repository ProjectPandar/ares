//! "Add instance" / "Remove instance" on the plater session (`Plater::increase_instances` /
//! `decrease_instances`, `Plater.cpp:19289-19364` at v2.5.0-beta `345efec64`; see
//! `docs/architecture/web-plater-instance-copies.md`). A copy is `add_instance(offset, scaling,
//! rotation, mirror)` of the last instance's components at the last offset + `(i·o, i·o, 0)`,
//! `o = 0.05 × 220` = 11 on the Ender-3 (`GLCanvas3D.cpp:6195-6199`); the plate then slices like
//! Native slicing the project the GUI exports.

use ares_core::{PlaterSession, SliceError};

use super::{SINGLE, assert_plate_like_native, load, rewrite_archive};
use crate::runner::repo_root;

/// The instance's column-major matrix.
fn matrix(session: &PlaterSession, instance: usize) -> [f64; 16] {
    session
        .instance_transform(0, instance)
        .unwrap()
        .to_column_major()
}

/// The cube's matrix moved by `(by, by, 0)`.
fn moved(cube: [f64; 16], by: f64) -> [f64; 16] {
    let mut expected = cube;
    expected[12] += by;
    expected[13] += by;
    expected
}

fn assert_close(actual: [f64; 16], expected: [f64; 16]) {
    assert!(
        actual
            .iter()
            .zip(&expected)
            .all(|(a, e)| (a - e).abs() < 1e-9),
        "{actual:?} != {expected:?}"
    );
}

/// `SINGLE` with its cube's build item rewritten by `from` -> `to`.
fn single_with(from: &str, to: &str) -> PlaterSession {
    let input = rewrite_archive(
        &std::fs::read(repo_root().join(SINGLE)).unwrap(),
        |name, text| {
            (name == "3D/3dmodel.model").then(|| {
                assert!(text.contains(from), "no {from}");
                text.replace(from, to)
            })
        },
    );
    PlaterSession::load(input).unwrap()
}

#[test]
fn plater_add_instance_places_copies_beside_the_last_instance_like_native() {
    let mut session = load(SINGLE);
    let cube = matrix(&session, 0);
    session.increase_instances(0, 2).unwrap();
    assert_eq!(matrix(&session, 1), moved(cube, 11.0));
    assert_eq!(matrix(&session, 2), moved(cube, 22.0));
    assert_eq!(session.plates()[0].instances, vec![(0, 0), (0, 1), (0, 2)]);
    assert_plate_like_native("instances-two-copies", &session, 1);
}

#[test]
fn plater_remove_instance_drops_the_last_copy_like_native() {
    let mut session = load(SINGLE);
    let cube = matrix(&session, 0);
    session.increase_instances(0, 2).unwrap();
    session.decrease_instances(0, 1).unwrap();
    assert_eq!(session.project().objects()[0].instances().len(), 2);
    assert_eq!(matrix(&session, 1), moved(cube, 11.0));
    assert_plate_like_native("instances-copy-removed", &session, 1);
}

#[test]
fn plater_add_instance_composes_a_rotated_mirrored_copy_like_native() {
    // R_z(30°) · diag(-1, 1, 1): `extract_rotation_scale` keeps the X flip, the largest trace, so
    // the copy takes rotation (0, 0, 30°), scaling factor (1, 1, 1) and mirror (-1, 1, 1)
    // (`Geometry.cpp:431-454, 471-474, 513-517, 550-554`), which compose back to the cube's
    // linear part through `set_scaling_factor`, `set_rotation` and `set_mirror`.
    let mut session = single_with(
        "transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"",
        "transform=\"-0.866025404 -0.5 0 -0.5 0.866025404 0 0 0 1 110 110 5\"",
    );
    let cube = matrix(&session, 0);
    // The components the port extracts, before and after the copy.
    for instance in [0, 1] {
        if instance == 1 {
            session.increase_instances(0, 1).unwrap();
        }
        let [rotation, scaling, mirror] = session.transformation_components(0, instance).unwrap();
        let near = |a: [f64; 3], e: [f64; 3]| a.iter().zip(&e).all(|(a, e)| (a - e).abs() < 1e-9);
        assert!(
            near(rotation, [0.0, 0.0, std::f64::consts::FRAC_PI_6]),
            "{rotation:?}"
        );
        assert!(near(scaling, [1.0, 1.0, 1.0]), "{scaling:?}");
        assert_eq!(mirror, [-1.0, 1.0, 1.0]);
    }
    assert_close(matrix(&session, 1), moved(cube, 11.0));
    assert_plate_like_native("instances-rotated-mirrored-copy", &session, 1);
}

#[test]
fn plater_instance_edits_follow_upstream_enabling_rules() {
    // `can_decrease_instances`: more than one instance (`Plater.cpp:15109-15118`).
    let mut session = load(SINGLE);
    assert!(matches!(
        session.decrease_instances(0, 1),
        Err(SliceError::InvalidInput(_))
    ));
    // `can_increase_instances`: every instance printable (`:15097-15107`).
    let mut unprintable = single_with("printable=\"1\"", "printable=\"0\"");
    assert!(matches!(
        unprintable.increase_instances(0, 1),
        Err(SliceError::InvalidInput(_))
    ));
}
