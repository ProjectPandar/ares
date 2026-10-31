// SPDX-License-Identifier: MPL-2.0
// Eigen algorithms copyright (C) Benoit Jacob and Gael Guennebaud.
// Mozilla Public License 2.0: https://mozilla.org/MPL/2.0/
// Geometry.cpp:420-426/471-474/754-777, using Eigen's computeRotationScaling and
// eulerAngles(2,1,0) evaluation order.
use crate::Transform3d;
use crate::geometry::{Matrix3 as Matrix, compute_rotation_scaling};

pub(super) fn difference(from: Transform3d, to: Transform3d) -> f64 {
    let from = euler(rotation(from));
    let to = euler(rotation(to));
    let axis = |angle: f64, index: usize| {
        let mut q = [0.0; 4];
        q[0] = (angle * 0.5).cos();
        q[index + 1] = (angle * 0.5).sin();
        q
    };
    let mut q = product(axis(to[2], 2), axis(to[1], 1));
    for next in [
        axis(to[0], 0),
        axis(-from[0], 0),
        axis(-from[1], 1),
        axis(-from[2], 2),
    ] {
        q = product(q, next);
    }
    let norm = (q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    let angle = 2.0 * norm.atan2(q[0].abs());
    let axis_z = if q[0] < 0.0 { -q[3] } else { q[3] };
    if axis_z < 0.0 { -angle } else { angle }
}

fn product(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] + a[2] * b[0] + a[3] * b[1] - a[1] * b[3],
        a[0] * b[3] + a[3] * b[0] + a[1] * b[2] - a[2] * b[1],
    ]
}

fn euler(m: Matrix) -> [f64; 3] {
    let mut z = m[1][0].atan2(m[0][0]);
    let c2 = (m[2][2] * m[2][2] + m[2][1] * m[2][1]).sqrt();
    let y = if z < 0.0 {
        z += std::f64::consts::PI;
        (-m[2][0]).atan2(-c2)
    } else {
        (-m[2][0]).atan2(c2)
    };
    let x = (z.sin() * m[0][2] - z.cos() * m[1][2]).atan2(z.cos() * m[1][1] - z.sin() * m[0][1]);
    [x, y, z]
}

fn rotation(transform: Transform3d) -> Matrix {
    let m = transform.as_matrix();
    compute_rotation_scaling(std::array::from_fn(|r| std::array::from_fn(|c| m[r][c]))).0
}
