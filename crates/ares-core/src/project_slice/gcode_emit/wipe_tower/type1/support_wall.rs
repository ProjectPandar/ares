//! `WipeTower.cpp:4248–4327`: generated Type1 wall, entry gaps and rib origin.
use super::super::geometry::{BoxCoordinates, Shape, gaps};
use super::{generator::Generator, writer::Writer};
use crate::geometry::{ClipperError, Polygon};

impl Generator<'_> {
    pub(super) fn support_wall(
        &mut self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        speed: f32,
        extrude: bool,
    ) -> Result<Polygon, ClipperError> {
        let shape = Shape {
            width: self.plan.width,
            depth: self.plan.depth,
            height: self.plan.height,
            z: self.z,
            y_shift: self.y_shift,
            rib_width: self.plan.rib_width,
            rib_length: self.plan.rib_length,
            fillet: self.settings.process.print.wipe_tower_fillet_wall.0,
        };
        let wall = shape.wall_polygon(bounds, self.plan.rib, self.scale)?;
        if !extrude {
            return Ok(wall);
        }
        let (mut paths, inserted) = gaps::split(
            &wall,
            if self.settings.process.print.prime_tower_skip_points.0 {
                &self.wall_gaps
            } else {
                &[]
            },
            self.plan.width,
            (2.5 * f64::from(self.plan.perimeter)) as f32,
            self.scale,
        );
        let filament = &self.filaments[self.current];
        writer.paths(
            &mut paths,
            speed,
            (filament.retract_length, filament.retract_speed * 60.0),
            self.scale,
        );
        if self.index == 0 {
            let x = paths
                .iter()
                .flat_map(|p| p.points())
                .map(|p| p.x())
                .min()
                .unwrap();
            let y = paths
                .iter()
                .flat_map(|p| p.points())
                .map(|p| p.y())
                .min()
                .unwrap();
            let factor = self.scale.factor() as f32;
            self.rib_offset = [-(x as f32 * factor), -(y as f32 * factor)];
        }
        Ok(inserted)
    }
}
