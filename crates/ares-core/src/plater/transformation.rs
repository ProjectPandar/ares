//! The `Geometry::Transformation` edits the plater applies (`Geometry.cpp:395-587, 640-645`
//! at v2.5.0-beta `345efec64`): a matrix split into rotation and scale by Eigen's
//! `computeRotationScaling`, a mirror kept on the axis needing the least rotation.

use super::eigen::{euler_angles_210, rotation_transform, scale_transform};
use crate::Transform3d;
use crate::geometry::{Matrix3, compute_rotation_scaling, jacobi_svd, multiply_transpose};

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
pub(crate) fn rotation(transform: Transform3d) -> [f64; 3] {
    euler_angles(rotation_scale(transform).0)
}

/// `Transformation::set_rotation(const Vec3d&)` (`Geometry.cpp:491-496`).
pub(crate) fn with_rotation(transform: Transform3d, angles: [f64; 3]) -> Transform3d {
    affine(
        rotated(angles, rotation_scale(transform).1),
        offset(transform),
    )
}

/// `ModelInstance::rotate(Matrix3d)` (`Model.hpp:1483-1487`): compose with
/// the current rotation, preserving the original instance offset and scale.
pub(super) fn rotated_instance(transform: Transform3d, matrix: Matrix3) -> Transform3d {
    let composed = affine(multiply(matrix, rotation_scale(transform).0), [0.; 3]);
    with_rotation(transform, rotation(composed))
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

/// `Transformation::get_matrix_no_scaling_factor` (`Geometry.cpp:647-652`) through
/// `reset_scaling_factor` (`:624-628`) and `TransformationSVD` (`:779-791`):
/// `T · U · Vᵀ · mirror`, a mirror taken out on X before the SVD and put back after it.
pub(super) fn without_scaling_factor(transform: Transform3d) -> Transform3d {
    let mut m = linear(transform);
    let mirror = determinant(m) < 0.0;
    if mirror {
        for row in &mut m {
            row[0] = -row[0];
        }
    }
    let (u, _, v) = jacobi_svd(m);
    let mut rotation = multiply_transpose(u, v);
    if mirror {
        for row in &mut rotation {
            row[0] = -row[0];
        }
    }
    affine(rotation, offset(transform))
}

/// `a * b`.
fn multiply(a: Matrix3, b: Matrix3) -> Matrix3 {
    std::array::from_fn(|i| {
        std::array::from_fn(|j| a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j])
    })
}

/// Eigen's 3x3 `inverse()` (`LU/InverseImpl.h`, `compute_inverse_size3_helper`): the
/// transposed cofactors over the determinant along the first column.
fn inverse(m: Matrix3) -> Matrix3 {
    let cofactor = |i: usize, j: usize| {
        let (i1, i2, j1, j2) = ((i + 1) % 3, (i + 2) % 3, (j + 1) % 3, (j + 2) % 3);
        m[i1][j1] * m[i2][j2] - m[i1][j2] * m[i2][j1]
    };
    let det = cofactor(0, 0) * m[0][0] + cofactor(1, 0) * m[1][0] + cofactor(2, 0) * m[2][0];
    let invdet = 1.0 / det;
    std::array::from_fn(|i| std::array::from_fn(|j| cofactor(j, i) * invdet))
}

/// `Selection::synchronize_unselected_instances` for one unselected instance `j` of the scaled
/// instance `i` (`Selection.cpp:3082-3088`): the linear part `(old_j · old_i⁻¹) · curr_i`,
/// the offset kept with Z from `curr_i`.
pub(super) fn synchronized(
    old_j: Transform3d,
    old_i: Transform3d,
    curr_i: Transform3d,
) -> Transform3d {
    let linear_j = multiply(
        multiply(linear(old_j), inverse(linear(old_i))),
        linear(curr_i),
    );
    let mut offset_j = offset(old_j);
    offset_j[2] = offset(curr_i)[2];
    affine(linear_j, offset_j)
}

/// `Transformation::set_offset(const Vec3d&)`.
pub(super) fn with_offset(transform: Transform3d, offset: [f64; 3]) -> Transform3d {
    affine(linear(transform), offset)
}

/// `Transformation::get_offset`.
pub(super) fn get_offset(transform: Transform3d) -> [f64; 3] {
    offset(transform)
}

/// `Transformation::get_scaling_factor` (`Geometry.cpp:513-517`): the extracted scale's diagonal,
/// without signs.
pub(super) fn scaling_factor(transform: Transform3d) -> [f64; 3] {
    let scale = rotation_scale(transform).1;
    [0, 1, 2].map(|axis| scale[axis][axis].abs())
}

/// `Transformation::set_scaling_factor(const Vec3d&)` (`Geometry.cpp:528-535`):
/// `extract_rotation_matrix(m) * scale_transform(scaling_factor)`, the offset kept.
pub(super) fn with_scaling_factor(transform: Transform3d, scaling_factor: [f64; 3]) -> Transform3d {
    let rotation = rotation_scale(transform).0;
    let scale = linear(scale_transform(scaling_factor));
    affine(multiply(rotation, scale), offset(transform))
}

/// `Transformation::get_mirror` (`Geometry.cpp:550-554`): each extracted scale's sign.
pub(super) fn mirror(transform: Transform3d) -> [f64; 3] {
    let scale = rotation_scale(transform).1;
    [0, 1, 2].map(|axis| scale[axis][axis] / scale[axis][axis].abs())
}

/// `Transformation::set_mirror(const Vec3d&)` (`Geometry.cpp:565-587`): a zero component reads as
/// 1, any other as its sign; each extracted scale whose sign disagrees is flipped.
pub(super) fn with_mirror(transform: Transform3d, mirror: [f64; 3]) -> Transform3d {
    let copy = mirror.map(|value| {
        let abs = value.abs();
        if abs == 0.0 {
            1.0
        } else if abs != 1.0 {
            value / abs
        } else {
            value
        }
    });
    let (rotation, mut scale) = rotation_scale(transform);
    for axis in 0..3 {
        if scale[axis][axis] * copy[axis] < 0.0 {
            scale[axis][axis] = -scale[axis][axis];
        }
    }
    affine(multiply(rotation, scale), offset(transform))
}
