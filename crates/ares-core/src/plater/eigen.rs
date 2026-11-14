//! The Eigen 5.0.1 geometry `Geometry::rotation_transform`,
//! `translation_transform` and `Transformation::get_rotation_by_quaternion`
//! build on (`Geometry.cpp:335-359, 481-489`; Eigen is `deps/Eigen`,
//! 5.0.1), in the operation order of an x86-64 SSE2 build.

use crate::Transform3d;

/// Eigen `Quaternion<double>`.
#[derive(Clone, Copy, Debug)]
struct Quaternion {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Quaternion {
    /// `QuaternionBase::operator=(const AngleAxis&)` (`Quaternion.h:617-624`).
    fn from_angle_axis(angle: f64, axis: [f64; 3]) -> Self {
        let half = 0.5 * angle;
        let sin = half.sin();
        Self {
            x: sin * axis[0],
            y: sin * axis[1],
            z: sin * axis[2],
            w: half.cos(),
        }
    }

    /// `quat_product` for `double` with SSE2 (`arch/Geometry_SIMD.h:87-127`):
    /// two packets per operand and `paddsub`.
    fn mul(self, b: Self) -> Self {
        let a = self;
        Self {
            x: (a.w * b.x + a.y * b.z) - (a.z * b.y - a.x * b.w),
            y: (a.w * b.y + a.y * b.w) + (a.z * b.x - a.x * b.z),
            z: (a.w * b.z - a.y * b.x) + (a.z * b.w + a.x * b.y),
            w: (a.w * b.w - a.y * b.y) - (a.z * b.z + a.x * b.x),
        }
    }

    /// `QuaternionBase::toRotationMatrix` (`Quaternion.h:646-678`).
    fn to_rotation_matrix(self) -> [[f64; 3]; 3] {
        let tx = 2.0 * self.x;
        let ty = 2.0 * self.y;
        let tz = 2.0 * self.z;
        let twx = tx * self.w;
        let twy = ty * self.w;
        let twz = tz * self.w;
        let txx = tx * self.x;
        let txy = ty * self.x;
        let txz = tz * self.x;
        let tyy = ty * self.y;
        let tyz = tz * self.y;
        let tzz = tz * self.z;
        [
            [1.0 - (tyy + tzz), txy - twz, txz + twy],
            [txy + twz, 1.0 - (txx + tzz), tyz - twx],
            [txz - twy, tyz + twx, 1.0 - (txx + tyy)],
        ]
    }

    /// `quaternionbase_assign_impl<Other, 3, 3>` (`Quaternion.h:883-914`).
    fn from_rotation_matrix(m: [[f64; 3]; 3]) -> Self {
        let mut t = m[0][0] + m[1][1] + m[2][2];
        if t > 0.0 {
            t = (t + 1.0).sqrt();
            let w = 0.5 * t;
            t = 0.5 / t;
            return Self {
                x: (m[2][1] - m[1][2]) * t,
                y: (m[0][2] - m[2][0]) * t,
                z: (m[1][0] - m[0][1]) * t,
                w,
            };
        }
        let mut i = 0;
        if m[1][1] > m[0][0] {
            i = 1;
        }
        if m[2][2] > m[i][i] {
            i = 2;
        }
        let j = (i + 1) % 3;
        let k = (j + 1) % 3;
        t = (m[i][i] - m[j][j] - m[k][k] + 1.0).sqrt();
        let mut coeffs = [0.0; 3];
        coeffs[i] = 0.5 * t;
        t = 0.5 / t;
        let w = (m[k][j] - m[j][k]) * t;
        coeffs[j] = (m[j][i] + m[i][j]) * t;
        coeffs[k] = (m[k][i] + m[i][k]) * t;
        Self {
            x: coeffs[0],
            y: coeffs[1],
            z: coeffs[2],
            w,
        }
    }

    /// `coeffs().normalize()`: the SSE2 `squaredNorm` reduction pairs
    /// `(x, y)` with `(z, w)`.
    fn normalized(self) -> Self {
        let squared = (self.x * self.x + self.z * self.z) + (self.y * self.y + self.w * self.w);
        if squared <= 0.0 {
            return self;
        }
        let norm = squared.sqrt();
        Self {
            x: self.x / norm,
            y: self.y / norm,
            z: self.z / norm,
            w: self.w / norm,
        }
    }
}

/// `Transform3d::Identity()` with `linear() *= rotation` (`Transform::rotate`).
fn rotation_matrix_transform(rotation: [[f64; 3]; 3]) -> Transform3d {
    let identity = Transform3d::IDENTITY;
    let mut rows = [[0.0; 4]; 4];
    for (row, values) in rotation.iter().enumerate() {
        rows[row][..3].copy_from_slice(values);
    }
    rows[3][3] = 1.0;
    identity.then(Transform3d::from_rows(rows))
}

/// `Geometry::rotation_transform` (`Geometry.cpp:348-359`): the quaternion
/// `AngleAxis(z, Z) * AngleAxis(y, Y) * AngleAxis(x, X)`.
pub(super) fn rotation_transform(rotation: [f64; 3]) -> Transform3d {
    let q = Quaternion::from_angle_axis(rotation[2], [0.0, 0.0, 1.0])
        .mul(Quaternion::from_angle_axis(rotation[1], [0.0, 1.0, 0.0]))
        .mul(Quaternion::from_angle_axis(rotation[0], [1.0, 0.0, 0.0]));
    rotation_matrix_transform(q.to_rotation_matrix())
}

/// `Geometry::rotation_from_two_vectors` (`Geometry.cpp:416-424`) and
/// Eigen `Quaternion::setFromTwoVectors` / `AngleAxis::operator=`.
pub(super) fn rotation_from_two_vectors(
    from: [f64; 3],
    to: [f64; 3],
) -> ([f64; 3], f64, [[f64; 3]; 3]) {
    let norm = |a: [f64; 3]| (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let from_norm = norm(from);
    let to_norm = norm(to);
    let a = from.map(|v| v / from_norm);
    let b = to.map(|v| v / to_norm);
    let c = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let q = if c < -1. + 1e-12 {
        let (_, _, v) = crate::geometry::jacobi_svd([a, b, [0.; 3]]);
        let w2 = (1. + c.max(-1.)) * 0.5;
        let scale = (1. - w2).sqrt();
        Quaternion {
            x: v[0][2] * scale,
            y: v[1][2] * scale,
            z: v[2][2] * scale,
            w: w2.sqrt(),
        }
    } else {
        let axis = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let s = ((1. + c) * 2.).sqrt();
        let inverse = 1. / s;
        Quaternion {
            x: axis[0] * inverse,
            y: axis[1] * inverse,
            z: axis[2] * inverse,
            w: s * 0.5,
        }
    };
    let matrix = q.to_rotation_matrix();
    let q = Quaternion::from_rotation_matrix(matrix);
    let n = norm([q.x, q.y, q.z]);
    let (axis, angle) = if n == 0. {
        ([1., 0., 0.], 0.)
    } else {
        let divisor = if q.w < 0. { -n } else { n };
        (
            [q.x / divisor, q.y / divisor, q.z / divisor],
            2. * n.atan2(q.w.abs()),
        )
    };
    (axis, angle, matrix)
}

pub(super) fn angle_axis_euler(angle: f64, axis: [f64; 3]) -> [f64; 3] {
    let angles = euler_angles_210(Quaternion::from_angle_axis(angle, axis).to_rotation_matrix());
    [angles[2], angles[1], angles[0]]
}

/// `Geometry::scale_transform(const Vec3d&)` (`Geometry.cpp:377-392`).
pub(super) fn scale_transform(scale: [f64; 3]) -> Transform3d {
    let mut rows = *Transform3d::IDENTITY.as_matrix();
    for (axis, value) in scale.into_iter().enumerate() {
        rows[axis][axis] = value;
    }
    Transform3d::from_rows(rows)
}

/// `Geometry::translation_transform` (`Geometry.cpp:335-346`).
pub(super) fn translation_transform(translation: [f64; 3]) -> Transform3d {
    let mut rows = *Transform3d::IDENTITY.as_matrix();
    for (row, value) in translation.into_iter().enumerate() {
        rows[row][3] = value;
    }
    Transform3d::from_rows(rows)
}

/// `Transformation::get_rotation_by_quaternion` (`Geometry.cpp:481-489`):
/// `eulerAngles(2, 1, 0)` of the normalized quaternion, X and Z swapped.
pub(super) fn rotation_by_quaternion(transform: Transform3d) -> [f64; 3] {
    let m = transform.as_matrix();
    let linear = std::array::from_fn(|row| std::array::from_fn(|column| m[row][column]));
    let rotation = Quaternion::from_rotation_matrix(linear)
        .normalized()
        .to_rotation_matrix();
    let angles = euler_angles_210(rotation);
    [angles[2], angles[1], angles[0]]
}

/// `numext::hypot` (`Core/MathFunctionsImpl.h:150-170`): `p * sqrt(1 + (q / p)^2)`.
fn hypot(x: f64, y: f64) -> f64 {
    let (x, y) = (x.abs(), y.abs());
    if x.is_infinite() || y.is_infinite() {
        return f64::INFINITY;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    let p = x.max(y);
    if p == 0.0 {
        return 0.0;
    }
    let qp = x.min(y) / p;
    p * (1.0 + qp * qp).sqrt()
}

/// `MatrixBase::eulerAngles(2, 1, 0)` (`EulerAngles.h:135-197`), the
/// Tait-Bryan branch with `odd = 1`, `i = 2`, `j = 1`, `k = 0`.
pub(super) fn euler_angles_210(m: [[f64; 3]; 3]) -> [f64; 3] {
    let (i, j, k) = (2, 1, 0);
    let mut res = [0.0; 3];
    res[0] = m[j][k].atan2(m[k][k]);
    let c2 = hypot(m[i][i], m[i][j]);
    if res[0] < 0.0 {
        res[0] += std::f64::consts::PI;
        res[1] = (-m[i][k]).atan2(-c2);
    } else {
        res[1] = (-m[i][k]).atan2(c2);
    }
    let s1 = res[0].sin();
    let c1 = res[0].cos();
    res[2] = (s1 * m[k][i] - c1 * m[j][i]).atan2(c1 * m[j][j] - s1 * m[k][j]);
    res
}
