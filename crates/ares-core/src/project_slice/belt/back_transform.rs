//! `BeltTransformPipeline` forward rotation (`BeltTransform.{hpp,cpp}`) and
//! `BeltBackTransform` (`GCode/BeltBackTransform.{hpp,cpp}`).

use crate::{BeltRotationAxis, Point3d, options::PrinterRemainingOptions, project::Transform3d};

use super::EPSILON;

/// Eigen `NumTraits<double>::dummy_precision()` used by `Transform3d::isApprox`.
const IS_APPROX_PRECISION: f64 = 1e-12;

/// `Geometry::deg2rad` (`Geometry.hpp:312-316`): `PI * angle / 180.0`.
#[inline]
pub(super) fn deg2rad(angle_deg: f64) -> f64 {
    std::f64::consts::PI * angle_deg / 180.0
}

/// Eigen `Transform3d::isApprox(Transform3d::Identity())` Frobenius-norm check.
pub(super) fn is_approx_identity(transform: Transform3d) -> bool {
    let matrix = transform.as_matrix();
    let mut diff_sq = 0.0;
    let mut norm_sq = 0.0;
    for (row_idx, row) in matrix.iter().enumerate() {
        for (col_idx, &value) in row.iter().enumerate() {
            let identity = if row_idx == col_idx { 1.0 } else { 0.0 };
            let diff = value - identity;
            diff_sq += diff * diff;
            norm_sq += value * value;
        }
    }
    diff_sq <= IS_APPROX_PRECISION * IS_APPROX_PRECISION * norm_sq.min(4.0)
}

/// `BeltTransformPipeline::build_forward_transform` (`BeltTransform.cpp:20-46`).
pub(in crate::project_slice) fn build_forward_transform(
    printer: &PrinterRemainingOptions,
) -> Transform3d {
    let axis = printer.belt_slice_rotation;
    let angle_deg = printer.belt_slice_rotation_angle.0;
    if axis == BeltRotationAxis::None || !angle_deg.is_finite() || angle_deg.abs() <= EPSILON {
        return Transform3d::IDENTITY;
    }
    let (sin, cos) = deg2rad(angle_deg).sin_cos();
    Transform3d::from_rows(match axis {
        BeltRotationAxis::X => [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cos, -sin, 0.0],
            [0.0, sin, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        BeltRotationAxis::Y => [
            [cos, 0.0, sin, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-sin, 0.0, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        BeltRotationAxis::Z => [
            [cos, -sin, 0.0, 0.0],
            [sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        BeltRotationAxis::None => unreachable!("checked above"),
    })
}

/// Reverses the pre-slice rotation from `BeltTransformPipeline::build_forward_transform`
/// (`GCode/BeltBackTransform.{hpp,cpp}`).
#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct BeltBackTransform {
    inverse: Option<Transform3d>,
}

impl BeltBackTransform {
    /// `BeltBackTransform::init_from_config` (`GCode/BeltBackTransform.cpp:8-24`).
    pub(in crate::project_slice) fn from_forward(belt_printer: bool, forward: Transform3d) -> Self {
        if !belt_printer || is_approx_identity(forward) {
            return Self { inverse: None };
        }
        let m = forward.as_matrix();
        let inverse = Transform3d::from_rows([
            [m[0][0], m[1][0], m[2][0], 0.0],
            [m[0][1], m[1][1], m[2][1], 0.0],
            [m[0][2], m[1][2], m[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        Self {
            inverse: Some(inverse),
        }
    }

    /// `BeltBackTransform::apply` (`GCode/BeltBackTransform.cpp:26-31`).
    #[inline]
    pub(in crate::project_slice) fn apply(self, pos: Point3d) -> Point3d {
        self.inverse
            .map_or(pos, |inverse| inverse.transform_point(pos))
    }
}
