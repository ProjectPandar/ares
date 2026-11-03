//! The plate menu's "Delete All" on the plater session (`Selection::remove_curr_plate`,
//! `Selection.cpp:507-525` at v2.5.0-beta `345efec64`; see
//! `docs/architecture/web-plater-plate-delete-all.md`): every instance the current plate holds or
//! overlaps is erased. An object with all its instances there goes (`notify_instance_removed(obj,
//! -1)`); otherwise its selected instances go, from the highest index down, telling no plate.

use ares_core::{ConfigOverrides, GenerationMetadata, PlaterSession, PlaterSlice, SliceError};

use super::{STRIDE, TWO_PLATES, assert_plate_like_native, export, load};
use crate::runner::{OrcaRunner, stages};

/// Two 10 mm cubes on an Ender-3: object 2 with two instances, object 3 with one.
const CUBES: &str = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";
const EMPTY: &str = "The supplied file couldn't be read because it's empty.";

#[test]
fn plater_plate_delete_all_removes_the_plates_whole_objects_like_native() {
    let mut session = load(TWO_PLATES);
    session.select_plate(1).unwrap();
    session.delete_plate_objects().unwrap();
    assert_eq!(session.project().objects().len(), 1);
    let plates = session.plates();
    assert_eq!(plates[0].instances, vec![(0, 0)]);
    assert!(plates[1].instances.is_empty());
    assert_plate_like_native("plate-delete-all-whole-objects", &session, 1);
}

/// Object 2's instance 1 alone sits on plate 2: it goes through the instance delete, so object 2
/// keeps its remaining instance at index 0 and every plate pair stays, plate 2's now dangling.
#[test]
fn plater_plate_delete_all_drops_only_the_plates_instance_and_keeps_the_pairs_like_native() {
    let mut session = load(CUBES);
    session.add_plate().unwrap();
    session
        .translate_instance(0, 1, [STRIDE, 0.0, 0.0])
        .unwrap();
    session.select_plate(1).unwrap();
    session.delete_plate_objects().unwrap();
    let objects = session.project().objects();
    assert_eq!(objects.len(), 2);
    assert_eq!(objects[0].instances().len(), 1);
    assert_eq!(objects[0].instances()[0].instance_id(), 0);
    let plates = session.plates();
    assert_eq!(plates[0].instances, vec![(0, 0), (1, 0)]);
    assert_eq!(plates[1].instances, vec![(0, 1)]);
    assert_plate_like_native("plate-delete-all-partial-object", &session, 1);
}

/// Plate 1 holds every instance of both objects: both go and the model is empty. Native's oracle is
/// its model-read error on the session's export (`Model.cpp:500-501`), which the session's slice
/// gives too.
#[test]
fn plater_plate_delete_all_of_every_instance_empties_the_model_like_native() {
    let mut session = load(CUBES);
    session.delete_plate_objects().unwrap();
    assert!(session.project().objects().is_empty());
    assert_eq!(session.plates().len(), 1);
    assert!(session.plates()[0].instances.is_empty());
    assert_empty_like_native("plate-delete-all-every-instance", &session);
}

/// Preservation control, no parity claim beyond plate 1: "Delete All" on an empty added plate
/// selects nothing and changes nothing (`Selection.cpp:513-515`).
#[test]
fn plater_plate_delete_all_on_an_empty_plate_changes_nothing() {
    let mut session = load(CUBES);
    session.add_plate().unwrap();
    session.select_plate(1).unwrap();
    // The whole session state, as the GUI export writes it.
    let before = export(&session);
    session.delete_plate_objects().unwrap();
    assert!(export(&session) == before, "the session changed");
    assert_plate_like_native("plate-delete-all-empty-plate", &session, 1);
}

/// Native `--slice 1` of the export fails the model read, and so does the session's slice.
fn assert_empty_like_native(label: &str, session: &PlaterSession) {
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_plater_plate(
            session,
            PlaterSlice {
                plate: Some(1),
                overrides: &ConfigOverrides::default(),
                validate_print: false,
            },
            GenerationMetadata::default(),
            ares_core::SlicerContext::new(0),
        ));
    assert!(matches!(actual, Err(SliceError::NoSuitableObjects(text)) if text == EMPTY));
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix(&format!("{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("native");
    std::fs::write(&path, export(session)).unwrap();
    std::fs::create_dir(&output).unwrap();
    let error = stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "1".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            path.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap_err();
    assert_eq!(
        error.captured.as_ref().unwrap().stderr,
        format!("{}: {EMPTY}\n", path.display()).as_bytes()
    );
}
