use super::*;

#[test]
fn plater_move_with_drop_slices_like_native() {
    let mut session = load(SINGLE);
    // A raised move drops back onto the bed (GLCanvas3D::do_move).
    session
        .translate_instance(0, 0, [35.0, -20.0, 7.0])
        .unwrap();
    assert_eq!(offset(&session, 0), [145.0, 90.0, 5.0]);
    session.set_instance_position(0, 0, 1, 95.5).unwrap();
    assert_eq!(offset(&session, 0), [145.0, 95.5, 5.0]);
    assert_plate_like_native("move", &session, 1);
}

#[test]
fn plater_move_to_second_plate_slices_like_native() {
    let mut session = load(TWO_PLATES);
    session
        .translate_instance(0, 0, [STRIDE + 50.0 - 110.0, 60.0 - 110.0, 0.0])
        .unwrap();
    let plates = session.plates();
    assert!(plates[0].instances.is_empty());
    assert_eq!(plates[1].instances, vec![(0, 0), (1, 0)]);
    assert_plate_like_native("second-plate", &session, 2);
}

#[test]
fn plater_add_and_delete_plates_slices_like_native() {
    let mut session = load(SINGLE);
    session.add_plate().unwrap();
    session
        .translate_instance(0, 0, [STRIDE, 0.0, 0.0])
        .unwrap();
    assert_eq!(session.plates()[1].instances, vec![(0, 0)]);
    session.add_plate().unwrap();
    // Deleting plate 1 moves the later plates and their instances up.
    assert!(session.delete_plate(0));
    let plates = session.plates();
    assert_eq!(plates.len(), 2);
    assert_eq!(plates[0].instances, vec![(0, 0)]);
    assert_eq!(offset(&session, 0), [110.0, 110.0, 5.0]);
    assert_plate_like_native("plates", &session, 1);
}

#[test]
fn plater_delete_plate_keeps_custom_gcodes_by_index_like_native() {
    let mut session = load(TWO_PLATES);
    session.add_plate().unwrap();
    // The second plate's instance moves up; Model::plates_custom_gcodes stays keyed by plate
    // index (PartPlateList::delete_plate does not touch it), so plate 1 keeps plate 1's
    // custom G-codes and the GUI export writes them unchanged (bbs_3mf.cpp:8469).
    assert!(session.delete_plate(0));
    assert_eq!(session.plates()[0].instances, vec![(1, 0)]);
    assert_eq!(offset(&session, 1), [110.0, 110.0, 5.0]);
    assert_plate_like_native("custom-gcodes-by-index", &session, 1);
}
