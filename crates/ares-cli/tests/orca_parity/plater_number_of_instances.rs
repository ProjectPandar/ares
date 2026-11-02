//! "Set number of instances" on the plater session (`Plater::set_number_of_copies`,
//! `Plater.cpp:19386-19406` at v2.5.0-beta `345efec64`; see
//! `docs/architecture/web-plater-number-of-instances.md`): the count reached through
//! `increase_instances` / `decrease_instances`, 0 removing the object (`Plater::priv::remove`,
//! `:10727-10747`). The plate then slices like Native slicing the project the GUI exports.

use ares_core::{PlaterSession, SliceError};

use super::{SINGLE, assert_plate_like_native, load};

/// Two 10 mm cubes on an Ender-3, by object: object 2 with two instances, object 3 with one.
const CUBES: &str = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";

fn matrix(session: &PlaterSession, instance: usize) -> [f64; 16] {
    session
        .instance_transform(0, instance)
        .unwrap()
        .to_column_major()
}

#[test]
fn plater_set_number_of_copies_adds_copies_like_native() {
    let mut session = load(SINGLE);
    let cube = matrix(&session, 0);
    session.set_number_of_copies(0, 3).unwrap();
    // The copies of `increase_instances(2)`: the cube moved by (11, 11) and (22, 22).
    for (instance, by) in [(1, 11.0), (2, 22.0)] {
        let mut expected = cube;
        expected[12] += by;
        expected[13] += by;
        assert_eq!(matrix(&session, instance), expected);
    }
    assert_plate_like_native("number-of-copies-three", &session, 1);
}

#[test]
fn plater_set_number_of_copies_removes_copies_like_native() {
    let mut session = load(SINGLE);
    let cube = matrix(&session, 0);
    session.set_number_of_copies(0, 3).unwrap();
    session.set_number_of_copies(0, 1).unwrap();
    assert_eq!(session.project().objects()[0].instances().len(), 1);
    assert_eq!(matrix(&session, 0), cube);
    assert_plate_like_native("number-of-copies-back-to-one", &session, 1);
}

#[test]
fn plater_set_number_of_copies_to_zero_removes_the_object_like_native() {
    let mut session = load(CUBES);
    session.set_number_of_copies(0, 0).unwrap();
    let ids: Vec<u32> = session.project().objects().iter().map(|o| o.id()).collect();
    assert_eq!(ids, [3]);
    assert_eq!(session.plates()[0].instances, vec![(0, 0)]);
    assert_plate_like_native("number-of-copies-zero", &session, 1);
}

#[test]
fn plater_set_number_of_copies_follows_upstream_bounds() {
    // `decrease_instances` returns silently on a single instance (`Plater.cpp:15109-15118`).
    let mut session = load(SINGLE);
    session.set_number_of_copies(0, 0).unwrap();
    assert_eq!(session.project().objects()[0].instances().len(), 1);
    // The dialog's range is 0..1000 (`:19394-19395`).
    assert!(matches!(
        session.set_number_of_copies(0, 1001),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn plater_remove_instance_down_to_none_removes_the_object() {
    // `decrease_instances(num)` with `num` at least the count calls `remove(obj_idx)`
    // (`Plater.cpp:19351-19353`).
    let mut session = load(CUBES);
    session.decrease_instances(0, 2).unwrap();
    let ids: Vec<u32> = session.project().objects().iter().map(|o| o.id()).collect();
    assert_eq!(ids, [3]);
}
