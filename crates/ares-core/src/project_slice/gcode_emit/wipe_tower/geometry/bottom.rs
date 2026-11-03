//! `Print.cpp:5225–5254::WipeTowerData::construct_mesh` bottom only;
//! rendering meshes are outside the byte-output boundary.
use super::super::data::Data;
use super::{rounding, scaled};
use crate::{
    ProcessWipeTowerWallType, ProjectSettings,
    geometry::{ClipperError, CoordinateScale, JoinType, Polygon, offset_paths},
};

impl Data {
    pub(in crate::project_slice::gcode_emit) fn bottom_polygon(
        &self,
        settings: &ProjectSettings,
        scale: CoordinateScale,
    ) -> Result<Polygon, ClipperError> {
        if self.width < 1e-4 || self.depth < 1e-4 || self.height < 1e-4 {
            return Ok(Polygon::new(Vec::new()));
        }
        let print = &settings.process.print;
        if print.wipe_tower_wall_type != ProcessWipeTowerWallType::Rib || self.rib_length < 1e-4 {
            // Preserve the source's asymmetric four corners, not a brim rectangle.
            return Ok(Polygon::new(
                [
                    [-self.brim_width, -self.brim_width],
                    [self.width + self.brim_width, 0.0],
                    [self.width + self.brim_width, self.depth + self.brim_width],
                    [0.0, self.depth],
                ]
                .map(|point| scaled(point, scale))
                .to_vec(),
            ));
        }
        let mut bottom = self.rib_section(scale);
        if print.wipe_tower_fillet_wall.0 {
            bottom = rounding::round_polygon(&bottom, scale);
        }
        let brim = offset_paths(
            std::slice::from_ref(&bottom),
            (f64::from(self.brim_width) / scale.factor()) as f32,
            JoinType::Miter,
            3.0,
        )?;
        if let Some(first) = brim.into_iter().next() {
            bottom = first;
        }
        // The rib-offset translation of the bottom (`Print.cpp:7236`) is not ported.
        Ok(bottom)
    }

    /// `WipeTower.cpp:1364–1403::rib_section`, distinct from Type2's wall union.
    fn rib_section(&self, scale: CoordinateScale) -> Polygon {
        let width = self.width;
        let depth = self.depth;
        let theta = (width / depth).atan();
        let w = self.rib_width / 2.0;
        let diagonal = (width * width + depth * depth).sqrt();
        let length = (self.rib_length - diagonal) / 2.0;
        let d1 = [width / diagonal, depth / diagonal];
        let d2 = [-width / diagonal, depth / diagonal];
        let corner = |p: [f32; 2], d: [f32; 2], l: f32, side: f32| {
            [p[0] + d[0] * l - d[1] * side, p[1] + d[1] * l + d[0] * side]
        };
        let mut points = [
            [0.0, w / theta.sin()],
            corner([0.0, 0.0], d1, -length, w),
            corner([0.0, 0.0], d1, -length, -w),
            [w / theta.cos(), 0.0],
            [width - w / theta.cos(), 0.0],
            corner([width, 0.0], d2, -length, w),
            corner([width, 0.0], d2, -length, -w),
            [width, w / theta.sin()],
            [width, depth - w / theta.sin()],
            corner([width, depth], d1, length, -w),
            corner([width, depth], d1, length, w),
            [width - w / theta.cos(), depth],
            [w / theta.cos(), depth],
            corner([0.0, depth], d2, length, -w),
            corner([0.0, depth], d2, length, w),
            [0.0, depth - w / theta.sin()],
        ]
        .map(|point| scaled(point, scale))
        .to_vec();
        points.dedup();
        if points.first() == points.last() {
            points.pop();
        }
        Polygon::new(points)
    }
}
