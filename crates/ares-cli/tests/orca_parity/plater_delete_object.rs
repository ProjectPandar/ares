//! The object menu's "Delete" of a whole object on the plater session
//! (`Plater::priv::delete_object_from_model`, `Plater.cpp:10750-10791` at v2.5.0-beta `345efec64`;
//! see `docs/architecture/web-plater-delete-object.md`): `Model::delete_object`, then
//! `notify_instance_removed(obj, -1)` drops the object's instances from the plates and renumbers the
//! later objects (`PartPlate.cpp:7396-7455`). The plate then slices like Native slicing the project
//! the GUI exports without the object.

use ares_core::SliceError;

use super::{TWO_PLATES, assert_plate_like_native, load};

/// Two 10 mm cubes on an Ender-3, by object: object 2 with two instances, object 3 with one.
const CUBES: &str = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";

#[test]
fn plater_delete_object_leaves_the_other_object_on_its_plate_like_native() {
    let mut session = load(CUBES);
    session.delete_object(0).unwrap();
    let ids: Vec<u32> = session.project().objects().iter().map(|o| o.id()).collect();
    assert_eq!(ids, [3]);
    assert_eq!(session.plates()[0].instances, vec![(0, 0)]);
    assert_plate_like_native("delete-object-same-plate", &session, 1);
}

#[test]
fn plater_delete_object_renumbers_the_objects_on_other_plates_like_native() {
    let mut session = load(TWO_PLATES);
    session.delete_object(0).unwrap();
    let plates = session.plates();
    assert!(plates[0].instances.is_empty());
    assert_eq!(plates[1].instances, vec![(0, 0)]);
    assert_plate_like_native("delete-object-other-plate", &session, 2);
}

#[test]
fn plater_delete_object_refuses_a_missing_object() {
    let mut session = load(CUBES);
    assert!(matches!(
        session.delete_object(5),
        Err(SliceError::InvalidInput(_))
    ));
}
