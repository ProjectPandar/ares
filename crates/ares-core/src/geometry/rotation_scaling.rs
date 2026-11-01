// SPDX-License-Identifier: MPL-2.0
// Eigen algorithms copyright (C) Benoit Jacob and Gael Guennebaud.
// Mozilla Public License 2.0: https://mozilla.org/MPL/2.0/
//! Eigen 5.0.1 `Transform::computeRotationScaling` (`Geometry/Transform.h:1052-1068`) over the
//! real 3x3 `JacobiSVD` (`SVD/JacobiSVD.h:700-805`, `misc/RealSvd2x2.h`,
//! `Jacobi/Jacobi.h:93-117`), in Eigen's evaluation order.

/// A row-major 3x3 matrix.
pub(crate) type Matrix3 = [[f64; 3]; 3];

const IDENTITY: Matrix3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// The full `JacobiSVD` of `linear`: `(U, singular values, V)`, the values descending.
pub(crate) fn jacobi_svd(linear: Matrix3) -> (Matrix3, [f64; 3], Matrix3) {
    let mut a = linear;
    let scale = a.iter().flatten().map(|v| v.abs()).fold(0.0_f64, f64::max);
    let scale = if scale == 0.0 { 1.0 } else { scale };
    for value in a.iter_mut().flatten() {
        *value /= scale;
    }
    let mut u = IDENTITY;
    let mut v = IDENTITY;
    let mut max_diagonal = (0..3).map(|i| a[i][i].abs()).fold(0.0_f64, f64::max);
    while !sweep(&mut a, &mut u, &mut v, &mut max_diagonal) {}
    let mut singular = [0.0; 3];
    for i in 0..3 {
        singular[i] = a[i][i].abs();
        if a[i][i] < 0.0 {
            for row in &mut u {
                row[i] = -row[i];
            }
        }
    }
    for value in &mut singular {
        *value *= scale;
    }
    for i in 0..3 {
        let mut pos = i;
        for j in i + 1..3 {
            if singular[j] > singular[pos] {
                pos = j;
            }
        }
        if singular[pos] == 0.0 {
            break;
        }
        if pos != i {
            singular.swap(i, pos);
            for (u_row, v_row) in u.iter_mut().zip(&mut v) {
                u_row.swap(i, pos);
                v_row.swap(i, pos);
            }
        }
    }
    (u, singular, v)
}

/// `computeRotationScaling`: the rotation `U' V*` and the scaling `V S' V*`, with the last
/// column of `U` and the last singular value negated when `U V*` is a reflection.
pub(crate) fn compute_rotation_scaling(linear: Matrix3) -> (Matrix3, Matrix3) {
    let (mut u, mut singular, v) = jacobi_svd(linear);
    if determinant(multiply_transpose(u, v)) < 0.0 {
        singular[2] = -singular[2];
        for row in &mut u {
            row[2] = -row[2];
        }
    }
    let scaled: Matrix3 = std::array::from_fn(|r| std::array::from_fn(|c| v[r][c] * singular[c]));
    (multiply_transpose(u, v), multiply_transpose(scaled, v))
}

fn block(a: Matrix3, p: usize, q: usize) -> ((f64, f64), (f64, f64)) {
    let t = a[p][p] + a[q][q];
    let d = a[q][p] - a[p][q];
    let (c, s) = if d.abs() < f64::MIN_POSITIVE {
        (1.0, 0.0)
    } else {
        let ratio = t / d;
        let norm = (1.0 + ratio * ratio).sqrt();
        (ratio / norm, 1.0 / norm)
    };
    let x = c * a[p][p] + s * a[q][p];
    let y = c * a[p][q] + s * a[q][q];
    let z = -s * a[p][q] + c * a[q][q];
    let (rc, rs) = if 2.0 * y.abs() < f64::MIN_POSITIVE {
        (1.0, 0.0)
    } else {
        let tau = (x - z) / (2.0 * y.abs());
        let w = (tau * tau + 1.0).sqrt();
        let t = if tau > 0.0 {
            1.0 / (tau + w)
        } else {
            1.0 / (tau - w)
        };
        let n = 1.0 / (t * t + 1.0).sqrt();
        let sign = if t > 0.0 { 1.0 } else { -1.0 };
        (n, -sign * (y / y.abs()) * t.abs() * n)
    };
    ((c * rc + s * rs, s * rc - c * rs), (rc, rs))
}

fn on_left(a: &mut Matrix3, p: usize, q: usize, (c, s): (f64, f64)) {
    let (before, after) = a.split_at_mut(p);
    for (x, y) in after[0].iter_mut().zip(&mut before[q]) {
        let (old_x, old_y) = (*x, *y);
        *x = c * old_x + s * old_y;
        *y = -s * old_x + c * old_y;
    }
}

fn on_right(a: &mut Matrix3, p: usize, q: usize, (c, s): (f64, f64)) {
    for row in a {
        let x = row[p];
        let y = row[q];
        row[p] = c * x - s * y;
        row[q] = s * x + c * y;
    }
}

/// `a * b^T`.
pub(crate) fn multiply_transpose(a: Matrix3, b: Matrix3) -> Matrix3 {
    std::array::from_fn(|r| {
        std::array::from_fn(|c| a[r][0] * b[c][0] + a[r][1] * b[c][1] + a[r][2] * b[c][2])
    })
}

fn determinant(m: Matrix3) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn sweep(a: &mut Matrix3, u: &mut Matrix3, v: &mut Matrix3, max_diagonal: &mut f64) -> bool {
    let mut finished = true;
    for p in 1..3 {
        for q in 0..p {
            let threshold = f64::MIN_POSITIVE.max(2.0 * f64::EPSILON * *max_diagonal);
            if a[p][q].abs() <= threshold && a[q][p].abs() <= threshold {
                continue;
            }
            finished = false;
            let (left, right) = block(*a, p, q);
            on_left(a, p, q, left);
            on_right(u, p, q, (left.0, -left.1));
            on_right(a, p, q, right);
            on_right(v, p, q, right);
            *max_diagonal = (*max_diagonal).max(a[p][p].abs().max(a[q][q].abs()));
        }
    }
    finished
}
