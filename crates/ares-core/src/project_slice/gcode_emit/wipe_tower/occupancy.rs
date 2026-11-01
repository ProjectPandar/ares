//! `Print.cpp:3020–3057`, `Print::first_layer_wipe_tower_corners`.
use super::Data;
use crate::{
    ProjectSettings,
    geometry::{CoordinateScale, Point},
};

impl Data {
    /// `placed`: the plate's stored position plus the plate origin (`Print.cpp:4489-4491`).
    pub(in crate::project_slice::gcode_emit) fn first_layer_corners(
        &self,
        settings: &ProjectSettings,
        scale: CoordinateScale,
        placed: [f64; 2],
    ) -> Vec<Point> {
        if self.changes.is_empty() {
            return Vec::new();
        }
        let bounds = self
            .first_layer_bounds
            .expect("generated tower retains first-layer polygon");
        let print = &settings.process.print;
        let (radius, stretch) = super::geometry::cone_base(
            print.prime_tower_width.0,
            f64::from(self.height),
            f64::from(self.depth),
            print.wipe_tower_cone_angle.0,
        );
        placed_corners(
            [
                [
                    scale.unscale(bounds.min().x()) + f64::from(self.rib_offset[0]),
                    scale.unscale(bounds.min().y()) + f64::from(self.rib_offset[1]),
                ],
                [
                    scale.unscale(bounds.max().x()) + f64::from(self.rib_offset[0]),
                    scale.unscale(bounds.max().y()) + f64::from(self.rib_offset[1]),
                ],
            ],
            (radius + f64::from(self.brim_width), stretch),
            (
                placed,
                print.wipe_tower_rotation_angle.0 * std::f64::consts::PI / 180.0,
            ),
            scale,
        )
    }
}

fn placed_corners(
    bounds: [[f64; 2]; 2],
    cone: (f64, f64),
    placement: ([f64; 2], f64),
    scale: CoordinateScale,
) -> Vec<Point> {
    let [min, max] = bounds;
    let mut points = vec![min, [max[0], min[1]], max, [min[0], max[1]]];
    let center = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
    let mut alpha = 0.0_f64;
    while alpha < 2.0 * std::f64::consts::PI {
        points.push([
            center[0] + cone.0 * (alpha.cos() / cone.1),
            center[1] + cone.0 * alpha.sin(),
        ]);
        alpha += std::f64::consts::PI / 20.0;
    }
    let (sin, cos) = placement.1.sin_cos();
    points
        .into_iter()
        .map(|[x, y]| {
            // Print.cpp constructs Point(double, double), which rounds in Point.hpp.
            Point::new(
                ((cos * x - sin * y + placement.0[0]) / scale.factor()).round() as i64,
                ((sin * x + cos * y + placement.0[1]) / scale.factor()).round() as i64,
            )
        })
        .collect()
}
