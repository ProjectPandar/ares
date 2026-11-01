//! Print.cpp:3565–3571,5171–5224 and Brim.cpp:651–659: live Type1 outer walls.
use super::super::Data;
use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::gcode_emit::conflict::Layer,
};

impl Data {
    /// `FakeWipeTower::pos` (`Print.hpp:793`, set at `Print.cpp:3628`): the plate's stored
    /// position plus the rib offset, plate-local (`Brim.cpp:682`).
    fn outer_wall_shift(&self, position: [f64; 2], scale: CoordinateScale) -> Point {
        let position = [
            position[0] as f32 + self.rib_offset[0],
            position[1] as f32 + self.rib_offset[1],
        ];
        Point::new(
            (f64::from(position[0]) / scale.factor()).round() as i64,
            (f64::from(position[1]) / scale.factor()).round() as i64,
        )
    }

    pub(in crate::project_slice::gcode_emit) fn brim_exclusions(
        &self,
        position: [f64; 2],
        scale: CoordinateScale,
    ) -> Vec<ExPolygon> {
        let Some((_, walls)) = self.outer_walls.first() else {
            return Vec::new();
        };
        let shift = self.outer_wall_shift(position, scale);
        walls
            .iter()
            .map(|wall| {
                let mut points = wall.points().to_vec();
                points.dedup();
                ExPolygon::new(
                    Polygon::new(
                        points
                            .into_iter()
                            .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y()))
                            .collect(),
                    ),
                    Vec::new(),
                )
            })
            .collect()
    }

    pub(in crate::project_slice::gcode_emit::wipe_tower) fn true_layers(
        &self,
        position: [f64; 2],
        scale: CoordinateScale,
    ) -> Vec<Layer> {
        let shift = self.outer_wall_shift(position, scale);
        let mut previous_z = 0.0;
        self.outer_walls
            .iter()
            .map(|(z, walls)| {
                let height = z - previous_z;
                previous_z = *z;
                let mut layer = Layer::new(z - height);
                for wall in walls {
                    layer.path(
                        wall.points()
                            .iter()
                            .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y())),
                        false,
                    );
                }
                layer
            })
            .collect()
    }
}
