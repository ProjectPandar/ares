//! The per-object brim on a plate after the first is plate-local: upstream emits it with
//! `set_origin(0, 0)` (`GCode.cpp:6053` at v2.5.0-beta `345efec64`) and the writer subtracts the
//! plate origin from every point (`Print.cpp:4010`, `GCodeWriter.cpp:1114`). See
//! `docs/architecture/brim-plate-origin.md`.

use super::{STRIDE, assert_plate_like_native, load};

/// Two 10 mm cubes on an Ender-3 with a per-object brim: object 2 with two instances, object 3 with
/// one.
const CUBES: &str = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";

#[test]
fn plater_brim_on_the_second_plate_slices_like_native() {
    let mut session = load(CUBES);
    session.add_plate().unwrap();
    session
        .translate_instance(1, 0, [STRIDE, 0.0, 0.0])
        .unwrap();
    assert_eq!(session.plates()[1].instances, vec![(1, 0)]);
    assert_plate_like_native("brim-second-plate", &session, 2);
}
