//! The `Geometry::Transformation` rotation edits arrange applies (`Geometry.cpp:395-516, 640-645`
//! at v2.5.0-beta `345efec64`): a matrix split into rotation and scale by Eigen's
//! `computeRotationScaling`, a mirror kept on the axis needing the least rotation.

use super::eigen::{euler_angles_210, rotation_transform};
use crate::Transform3d;
use crate::geometry::{Matrix3, compute_rotation_scaling};

/// `EPSILON` (`libslic3r.h:52`).
const EPSILON: f64 = 1e-4;

fn linear(transform: Transform3d) -> Matrix3 {
    let m = transform.as_matrix();
    std::array::from_fn(|r| std::array::from_fn(|c| m[r][c]))
}

fn offset(transform: Transform3d) -> [f64; 3] {
    let m = transform.as_matrix();
    [m[0][3], m[1][3], m[2][3]]
}

/// An affine transform of `linear` moved by `offset`.
fn affine(linear: Matrix3, offset: [f64; 3]) -> Transform3d {
    let mut rows = [[0.0; 4]; 4];
    for row in 0..3 {
        rows[row][..3].copy_from_slice(&linear[row]);
        rows[row][3] = offset[row];
    }
    rows[3][3] = 1.0;
    Transform3d::from_rows(rows)
}

/// `rotation_transform(angles) * Transform3d(scale)`: the affine product's linear part.
fn rotated(angles: [f64; 3], scale: Matrix3) -> Matrix3 {
    let r = linear(rotation_transform(angles));
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            r[i][0] * scale[0][j] + r[i][1] * scale[1][j] + r[i][2] * scale[2][j]
        })
    })
}

fn determinant(m: Matrix3) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// `extract_rotation_scale` (`Geometry.cpp:431-454`): a reflection flips each axis in turn and
/// keeps the rotation with the largest trace, the lowest axis on ties.
fn rotation_scale(transform: Transform3d) -> (Matrix3, Matrix3) {
    let m = linear(transform);
    if determinant(m) >= 0.0 {
        return compute_rotation_scaling(m);
    }
    let trace = |r: &Matrix3| r[0][0] + r[1][1] + r[2][2];
    let mut best = (m, m);
    for axis in 0..3 {
        let mut flipped = m;
        for row in &mut flipped {
            row[axis] = -row[axis];
        }
        let (rotation, mut scale) = compute_rotation_scaling(flipped);
        if axis == 0 || trace(&rotation) > trace(&best.0) + EPSILON {
            for row in &mut scale {
                row[axis] = -row[axis];
            }
            best = (rotation, scale);
        }
    }
    best
}

/// `extract_euler_angles(const Transform3d&)` (`Geometry.cpp:405-414`): the columns
/// normalized, then `eulerAngles(2, 1, 0)` with X and Z swapped.
fn euler_angles(mut m: Matrix3) -> [f64; 3] {
    for column in 0..3 {
        let squared =
            m[0][column] * m[0][column] + m[1][column] * m[1][column] + m[2][column] * m[2][column];
        if squared > 0.0 {
            let norm = squared.sqrt();
            for row in &mut m {
                row[column] /= norm;
            }
        }
    }
    let angles = euler_angles_210(m);
    [angles[2], angles[1], angles[0]]
}

/// `Transformation::get_rotation` (`Geometry.cpp:471-474`).
pub(super) fn rotation(transform: Transform3d) -> [f64; 3] {
    euler_angles(rotation_scale(transform).0)
}

/// `Transformation::set_rotation(const Vec3d&)` (`Geometry.cpp:491-496`).
pub(super) fn with_rotation(transform: Transform3d, angles: [f64; 3]) -> Transform3d {
    affine(
        rotated(angles, rotation_scale(transform).1),
        offset(transform),
    )
}

/// `Transformation::set_rotation(Axis, double)` (`Geometry.cpp:498-512`): the angle wrapped
/// into `[0, 2π]`, a full turn read as none.
pub(super) fn with_axis_rotation(transform: Transform3d, axis: usize, angle: f64) -> Transform3d {
    let two_pi = 2.0 * std::f64::consts::PI;
    let mut angle = angle;
    while angle < 0.0 {
        angle += two_pi;
    }
    while two_pi < angle {
        angle -= two_pi;
    }
    if (angle.abs() - two_pi).abs() < EPSILON {
        angle = 0.0;
    }
    let (current, scale) = rotation_scale(transform);
    let mut angles = euler_angles(current);
    angles[axis] = angle;
    affine(rotated(angles, scale), offset(transform))
}

/// `Transformation::get_matrix_no_offset` (`Geometry.cpp:640-645`).
pub(super) fn without_offset(transform: Transform3d) -> Transform3d {
    affine(linear(transform), [0.0; 3])
}

/// `Transformation::set_offset(const Vec3d&)`.
pub(super) fn with_offset(transform: Transform3d, offset: [f64; 3]) -> Transform3d {
    affine(linear(transform), offset)
}
