//! The instance transforms on the plater session: rotate with its drop, its pivot, scale and
//! mirror (see `plater_session.rs`).

use ares_core::PlaterSession;

use super::{SINGLE, assert_oracle_like_native, assert_plate_like_native, load, rewrite_archive};

#[path = "plater_mirror.rs"]
mod mirror;
#[path = "plater_rotation_pivot.rs"]
mod rotation_pivot;
#[path = "plater_scale.rs"]
mod scale;

/// `Selection::rotate` of a lone instance about its bounding-sphere centre
/// in world coordinates is `R · M` around that point; the cube's sphere
/// centre is its offset.
fn rotation_about_offset(rotations: &[[[f64; 3]; 3]], offset: [f64; 3]) -> [f64; 16] {
    let linear = rotations
        .iter()
        .fold(IDENTITY, |acc, rotation| multiply(rotation, &acc));
    let mut matrix = [0.0; 16];
    for column in 0..3 {
        for row in 0..3 {
            matrix[column * 4 + row] = linear[row][column];
        }
    }
    matrix[12..15].copy_from_slice(&offset);
    matrix[15] = 1.0;
    matrix
}

const IDENTITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn multiply(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| (0..3).map(|k| a[row][k] * b[k][column]).sum())
    })
}

fn about_x(degrees: f64) -> [[f64; 3]; 3] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [[1.0, 0.0, 0.0], [0.0, cos, -sin], [0.0, sin, cos]]
}

fn about_z(degrees: f64) -> [[f64; 3]; 3] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [[cos, -sin, 0.0], [sin, cos, 0.0], [0.0, 0.0, 1.0]]
}

fn assert_transform(session: &PlaterSession, expected: [f64; 16]) {
    let actual = session.instance_transform(0, 0).unwrap().to_column_major();
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual:?} != {expected:?}"
        );
    }
}

#[test]
fn plater_rotate_with_drop_slices_like_native() {
    let mut session = load(SINGLE);
    session
        .rotate_instance(0, 0, [0.0, 0.0, std::f64::consts::FRAC_PI_4])
        .unwrap();
    assert_transform(
        &session,
        rotation_about_offset(&[about_z(45.0)], [110.0, 110.0, 5.0]),
    );
    // A quarter turn about X keeps a flat bottom (a corner on the bed slices to an empty first
    // layer, a Native slicing error); the absolute field rotates by the change (X 0 -> 90) and
    // do_rotate drops the turned cube, whose half height is now 5√2, to the bed.
    session.set_absolute_rotation(0, 0, 0, 90.0).unwrap();
    let height = 5.0 * std::f64::consts::SQRT_2;
    assert_transform(
        &session,
        rotation_about_offset(&[about_z(45.0), about_x(90.0)], [110.0, 110.0, height]),
    );
    assert_plate_like_native("rotate", &session, 1);
}
