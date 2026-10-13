// SPDX-License-Identifier: MPL-2.0
// Eigen algorithms copyright (C) Benoit Jacob and Gael Guennebaud.
// Mozilla Public License 2.0: https://mozilla.org/MPL/2.0/
// Geometry.cpp:420-426/471-474/754-777, using Eigen's real JacobiSVD,
// RealSvd2x2, JacobiRotation and eulerAngles(2,1,0) evaluation order.
use crate::Transform3d;

type Matrix = [[f64; 3]; 3];
const IDENTITY: Matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

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
    let mut a: Matrix =
        std::array::from_fn(|r| std::array::from_fn(|c| transform.as_matrix()[r][c]));
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
    let candidate = multiply_transpose(u, v);
    if determinant(candidate) < 0.0 {
        for row in &mut u {
            row[2] = -row[2];
        }
    }
    multiply_transpose(u, v)
}

fn block(a: Matrix, p: usize, q: usize) -> ((f64, f64), (f64, f64)) {
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

fn on_left(a: &mut Matrix, p: usize, q: usize, (c, s): (f64, f64)) {
    let (before, after) = a.split_at_mut(p);
    for (x, y) in after[0].iter_mut().zip(&mut before[q]) {
        let (old_x, old_y) = (*x, *y);
        *x = c * old_x + s * old_y;
        *y = -s * old_x + c * old_y;
    }
}
fn on_right(a: &mut Matrix, p: usize, q: usize, (c, s): (f64, f64)) {
    for row in a {
        let x = row[p];
        let y = row[q];
        row[p] = c * x - s * y;
        row[q] = s * x + c * y;
    }
}
fn multiply_transpose(a: Matrix, b: Matrix) -> Matrix {
    std::array::from_fn(|r| {
        std::array::from_fn(|c| a[r][0] * b[c][0] + a[r][1] * b[c][1] + a[r][2] * b[c][2])
    })
}
fn determinant(m: Matrix) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn sweep(a: &mut Matrix, u: &mut Matrix, v: &mut Matrix, max_diagonal: &mut f64) -> bool {
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
