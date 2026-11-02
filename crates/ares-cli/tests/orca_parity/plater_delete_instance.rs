//! "Delete" of one instance on the plater session (`Plater::remove_selected` →
//! `ObjectList::del_subobject_from_object`, `GUI_ObjectList.cpp:2998-3073` at v2.5.0-beta
//! `345efec64`; see `docs/architecture/web-plater-delete-instance.md`):
//! `ModelObject::delete_instance` moves the later instances down one index, and no plate is
//! notified, so every plate pair stays, the now-dangling last one included. The plates then slice
//! like Native slicing the project the GUI exports.

use ares_core::SliceError;

use super::{STRIDE, assert_plate_like_native, export, load};

/// Two 10 mm cubes on an Ender-3, by object: object 2 with two instances, object 3 with one.
const CUBES: &str = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";

#[test]
fn plater_delete_instance_keeps_the_plate_pairs_like_native() {
    let mut session = load(CUBES);
    session.delete_instance(0, 0).unwrap();
    assert_eq!(session.project().objects()[0].instances().len(), 1);
    // The pair (0, 1) now names no instance; (0, 0) names the instance that moved down.
    assert_eq!(session.plates()[0].instances, vec![(0, 0), (0, 1), (1, 0)]);
    assert_plate_like_native("delete-instance-same-plate", &session, 1);
}

#[test]
fn plater_delete_instance_of_a_lone_instance_deletes_the_object() {
    let mut session = load(CUBES);
    session.delete_instance(1, 0).unwrap();
    let ids: Vec<u32> = session.project().objects().iter().map(|o| o.id()).collect();
    assert_eq!(ids, [2]);
    assert_eq!(session.plates()[0].instances, vec![(0, 0), (0, 1)]);
    assert_plate_like_native("delete-instance-lone", &session, 1);
}

#[test]
fn plater_delete_instance_refuses_a_missing_instance() {
    let mut session = load(CUBES);
    for (object, instance) in [(5, 0), (0, 2), (1, 1)] {
        assert!(matches!(
            session.delete_instance(object, instance),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

/// Cross-plate membership stays literal: plate 1 keeps (0, 0) while the instance now in slot 0 sits
/// on plate 2's area, and plate 2 keeps the dangling (0, 1). Each plate's export lists object 2, so
/// the import keeps it whole (`bbs_3mf.cpp:2215-2221`) and only the instance inside the plate's
/// volume prints: plate 1 prints object 3, plate 2 the moved cube.
#[test]
fn plater_delete_instance_keeps_cross_plate_pairs_like_native() {
    let mut session = load(CUBES);
    session.add_plate().unwrap();
    session
        .translate_instance(0, 1, [STRIDE, 0.0, 0.0])
        .unwrap();
    assert_eq!(session.plates()[1].instances, vec![(0, 1)]);
    session.delete_instance(0, 0).unwrap();
    let plates = session.plates();
    assert_eq!(plates[0].instances, vec![(0, 0), (1, 0)]);
    assert_eq!(plates[1].instances, vec![(0, 1)]);
    assert_plate_like_native("delete-instance-cross-plate", &session, 1);
    assert_plate_like_native("delete-instance-cross-plate-2", &session, 2);
}

/// The dangling pair is exported as upstream's writer emits it (`bbs_3mf.cpp:8580-8605`).
#[test]
fn plater_delete_instance_exports_the_dangling_pair_like_upstream() {
    let mut session = load(CUBES);
    session.delete_instance(0, 0).unwrap();
    let archive = export(&session);
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive)).unwrap();
    let mut settings = String::new();
    std::io::Read::read_to_string(
        &mut zip.by_name("Metadata/model_settings.config").unwrap(),
        &mut settings,
    )
    .unwrap();
    assert!(settings.contains(
        "<metadata key=\"object_id\" value=\"2\"/>\n      <metadata key=\"instance_id\" value=\"1\"/>\n      <metadata key=\"identify_id\" value=\"0\"/>"
    ));
}
