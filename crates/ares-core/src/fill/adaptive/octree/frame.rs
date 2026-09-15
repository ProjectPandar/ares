//! Octree frame rotations (`FillAdaptive.cpp:401-412`) and Ericson's SAT
//! triangle/AABB test (`FillAdaptive.cpp:42-160`).

const OCTREE_ROT: [f64; 3] = [
    5.0 * std::f64::consts::PI / 4.0,
    215.264f64.to_radians(),
    std::f64::consts::PI / 6.0,
];

/// `AngleAxisd(z, rot2) * AngleAxisd(y, rot1) * AngleAxisd(x, rot0)`
pub(super) fn octree_rotation_to_world() -> Matrix3 {
    mat_mul(
        mat_mul(rotate_axis(2, OCTREE_ROT[2]), rotate_axis(1, OCTREE_ROT[1])),
        rotate_axis(0, OCTREE_ROT[0]),
    )
}

/// `AngleAxisd(-rot0, x) * AngleAxisd(-rot1, y) * AngleAxisd(-rot2, z)`
pub(crate) fn octree_rotation_to_octree() -> Matrix3 {
    mat_mul(
        mat_mul(
            rotate_axis(0, -OCTREE_ROT[0]),
            rotate_axis(1, -OCTREE_ROT[1]),
        ),
        rotate_axis(2, -OCTREE_ROT[2]),
    )
}

pub(super) type Matrix3 = [[f64; 3]; 3];

pub(super) fn mat_mul(a: Matrix3, b: Matrix3) -> Matrix3 {
    let mut out = [[0.0; 3]; 3];
    for (r, row) in a.iter().enumerate() {
        for c in 0..3 {
            out[r][c] = row[0] * b[0][c] + row[1] * b[1][c] + row[2] * b[2][c];
        }
    }
    out
}

pub(crate) fn rotate_point(m: Matrix3, p: [f64; 3]) -> [f64; 3] {
    [dot(m[0], p), dot(m[1], p), dot(m[2], p)]
}

fn rotate_axis(axis: usize, angle: f64) -> Matrix3 {
    let (s, c) = angle.sin_cos();
    match axis {
        0 => [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]],
        1 => [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]],
        _ => [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]],
    }
}

pub(super) fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

pub(super) fn abs(a: [f64; 3]) -> [f64; 3] {
    a.map(f64::abs)
}

/// `FillAdaptive.cpp:42-160` — Ericson's SAT triangle/AABB test,
/// unrolled cross-axis form (`FillAdaptive.cpp:84-160`).
pub(super) fn triangle_aabb_intersects(
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    aabb: ([f64; 3], [f64; 3]),
) -> bool {
    let t_min = [
        a[0].min(b[0]).min(c[0]),
        a[1].min(b[1]).min(c[1]),
        a[2].min(b[2]).min(c[2]),
    ];
    let t_max = [
        a[0].max(b[0]).max(c[0]),
        a[1].max(b[1]).max(c[1]),
        a[2].max(b[2]).max(c[2]),
    ];
    if t_min[0] >= aabb.1[0]
        || t_max[0] <= aabb.0[0]
        || t_min[1] >= aabb.1[1]
        || t_max[1] <= aabb.0[1]
        || t_min[2] >= aabb.1[2]
        || t_max[2] <= aabb.0[2]
    {
        return false;
    }

    let center = [
        (aabb.0[0] + aabb.1[0]) * 0.5,
        (aabb.0[1] + aabb.1[1]) * 0.5,
        (aabb.0[2] + aabb.1[2]) * 0.5,
    ];
    let h = [
        aabb.1[0] - center[0],
        aabb.1[1] - center[1],
        aabb.1[2] - center[2],
    ];

    let t = [sub(b, a), sub(c, a), sub(c, b)];
    let ac = sub(a, center);
    let bc = sub(b, center);
    let cc = sub(c, center);

    let n = cross(t[0], t[1]);
    let s = dot(n, ac);
    let an = abs(n);
    let r0 = h[0] * an[0] + h[1] * an[1] + h[2] * an[2];
    if s.abs() >= r0 {
        return false;
    }

    let at = [abs(t[0]), abs(t[1]), abs(t[2])];

    // Each test: `if (r + |tc - d1| < |tc|) return false;`
    let sat = |d1: f64, d2: f64, r: f64| -> bool {
        let tc = (d1 + d2) * 0.5;
        !(r + (tc - d1).abs() < tc.abs())
    };

    // eX × t[0], eX × t[1], eX × t[2]
    for (ti, (p1, p2)) in [(0usize, (ac, cc)), (1, (ac, bc)), (2, (ac, bc))] {
        let d1 = t[ti][1] * p1[2] - t[ti][2] * p1[1];
        let d2 = t[ti][1] * p2[2] - t[ti][2] * p2[1];
        let r = (h[1] * at[ti][2] + h[2] * at[ti][1]).abs();
        if !sat(d1, d2, r) {
            return false;
        }
    }
    // eY × t[0..2]
    for (ti, (p1, p2)) in [(0usize, (ac, cc)), (1, (ac, bc)), (2, (ac, bc))] {
        let d1 = t[ti][2] * p1[0] - t[ti][0] * p1[2];
        let d2 = t[ti][2] * p2[0] - t[ti][0] * p2[2];
        let r = (h[0] * at[ti][2] + h[2] * at[ti][0]).abs();
        if !sat(d1, d2, r) {
            return false;
        }
    }
    // eZ × t[0..2]
    for (ti, (p1, p2)) in [(0usize, (ac, cc)), (1, (ac, bc)), (2, (ac, bc))] {
        let d1 = t[ti][0] * p1[1] - t[ti][1] * p1[0];
        let d2 = t[ti][0] * p2[1] - t[ti][1] * p2[0];
        let r = (h[1] * at[ti][0] + h[0] * at[ti][1]).abs();
        if !sat(d1, d2, r) {
            return false;
        }
    }
    true
}
