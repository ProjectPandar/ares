//! `Print.hpp:663–737::FakeWipeTower::getFakeExtrusionPathsFromWipeTower2`
//! and `Print.cpp:5177–5191` layer-bottom accumulation.
use super::super::conflict::Layer;
use super::Data;
use crate::{
    ProjectSettings,
    geometry::{CoordinateScale, Point},
};
use std::f64::consts::PI;

impl Data {
    pub(in crate::project_slice::gcode_emit) fn fake_layers(
        &self,
        settings: &ProjectSettings,
        scale: CoordinateScale,
        position: [f64; 2],
    ) -> Vec<Layer> {
        if !self.outer_walls.is_empty() {
            return self.true_layers(position, scale);
        }
        let print = &settings.process.print;
        let scale_value = |value: f32| (f64::from(value) / scale.factor()) as i64;
        let width = scale_value(self.width);
        let brim = scale_value(self.brim_width);
        let height_step = print.initial_layer_print_height.0 as f32;
        let angle = std::f32::consts::PI * print.wipe_tower_rotation_angle.0 as f32 / 180.0;
        let (sin, cos) = f64::from(angle).sin_cos();
        let shift = [
            scale_value(position[0] as f32),
            scale_value(position[1] as f32),
        ];
        let place = |p: Point| {
            Point::new(
                (cos * p.x() as f64 - sin * p.y() as f64).round() as i64 + shift[0],
                (cos * p.y() as f64 + sin * p.x() as f64).round() as i64 + shift[1],
            )
        };
        let (radius, stretch) = super::geometry::cone_base(
            f64::from(self.width),
            f64::from(self.height),
            f64::from(self.depth),
            f64::from(print.wipe_tower_cone_angle.0 as f32),
        );
        let center = [f64::from(self.width) / 2.0, f64::from(self.depth) / 2.0];
        let mut layers = Vec::new();
        let mut z = 0.0_f32;
        while z < self.height {
            let (min, max) = if z == 0.0 {
                ([-brim, -brim], [width, scale_value(self.depth)])
            } else {
                let index = self
                    .depth_profile
                    .windows(2)
                    .position(|pair| z >= pair[0].0 && z < pair[1].0)
                    .unwrap_or(self.depth_profile.len() - 1);
                let depth = scale_value(self.depth_profile[index].1);
                let lower = -depth / 2 + scale_value(self.depth_profile[0].1 / 2.0);
                ([0, lower], [width, lower + depth])
            };
            let mut layer = Layer::new(z);
            let point = |[x, y]: [i64; 2]| place(Point::new(x, y));
            layer.path(
                [min, [max[0], min[1]], max, [min[0], max[1]], min]
                    .into_iter()
                    .map(point),
                false,
            );
            let spacing = (3.0 / scale.factor()) as i64;
            let mut y = min[1] + spacing;
            while y < max[1] {
                layer.path([[min[0], y], [max[0], y]].into_iter().map(point), false);
                y += spacing;
            }
            if radius > 0.0 {
                add_cone(
                    &mut layer,
                    center,
                    (radius * f64::from(1.0 - z / self.height), stretch, None),
                    &place,
                    scale,
                );
            }
            if radius > 0.0 && z == 0.0 {
                add_brim(
                    &mut layer,
                    center,
                    (radius, stretch, self.brim_width),
                    &place,
                    scale,
                );
            }
            layers.push(layer);
            z += height_step;
        }
        layers
    }
}

fn add_brim(
    layer: &mut Layer,
    center: [f64; 2],
    (radius, stretch, mut brim): (f64, f64, f32),
    place: &impl Fn(Point) -> Point,
    scale: CoordinateScale,
) {
    while brim > 0.0 {
        add_cone(
            layer,
            center,
            (radius, stretch, Some(f64::from(brim))),
            place,
            scale,
        );
        brim -= 3.0;
    }
}
fn add_cone(
    layer: &mut Layer,
    center: [f64; 2],
    (radius, stretch, brim): (f64, f64, Option<f64>),
    place: &impl Fn(Point) -> Point,
    scale: CoordinateScale,
) {
    let mut points = Vec::new();
    let mut alpha = 0.0;
    while alpha < 2.01 * PI {
        let mut x = radius * alpha.cos() / stretch;
        let mut y = radius * alpha.sin();
        if let Some(brim) = brim {
            x *= 1.0 + stretch * brim / radius;
            y *= 1.0 + brim / radius;
        }
        points.push(place(Point::new(
            ((center[0] + x) / scale.factor()) as i64,
            ((center[1] + y) / scale.factor()) as i64,
        )));
        alpha += 2.0 * PI / 20.0;
    }
    layer.path(points.into_iter(), false);
}
