//! The G-code writer's plate offset (`GCodeWriter::m_x_offset`, `m_y_offset`, set by
//! `GCode::set_gcode_offset`, `Print.cpp:4000-4010` at v2.5.0-beta `345efec64`). See
//! `docs/architecture/gcode-plate-offset.md`.

use ares_core::{PlaterSession, ScaleEdit};

use super::{SINGLE, STRIDE, assert_plate_like_native, load};

fn set_size(session: &mut PlaterSession, object: usize, axis: usize, value: f64) {
    session
        .set_size(
            object,
            0,
            ScaleEdit {
                axis,
                value,
                uniform: false,
            },
        )
        .unwrap();
}

/// `orca_cli_ender3.3mf`'s cube at 12.5 x 12.5 x 250 mm, centred at `(110, 110)` plus `shift`.
fn tall_cube(shift: f64, plates: usize) -> PlaterSession {
    let mut session = load(SINGLE);
    for _ in 1..plates {
        session.add_plate().unwrap();
    }
    for (axis, value) in [12.5, 12.5, 250.0].into_iter().enumerate() {
        set_size(&mut session, 0, axis, value);
    }
    let at = session.instance_transform(0, 0).unwrap().to_column_major();
    session
        .translate_instance(0, 0, [110.0 + shift - at[12], 110.0 - at[13], 0.0])
        .unwrap();
    session
}

#[test]
fn plate_two_writes_its_moves_less_the_plate_origin_like_native() {
    // Native emits `unscale(p) + shift - extruder_offset` in the global frame and the writer
    // subtracts the plate origin (`GCodeWriter.cpp:1303-1314`); line 536 rounds to X116.892.
    let session = tall_cube(STRIDE, 2);
    assert_eq!(session.plates()[1].instances, vec![(0, 0)]);
    assert_plate_like_native("plate-offset-tall-cube-plate-two", &session, 2);
}

#[test]
fn plate_one_with_a_zero_offset_slices_like_native() {
    let session = tall_cube(0.0, 1);
    assert_plate_like_native("plate-offset-tall-cube-plate-one", &session, 1);
}
