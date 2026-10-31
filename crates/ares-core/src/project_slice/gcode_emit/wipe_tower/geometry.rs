//! `WipeTower2.cpp` rib/rectangle geometry; shared `WipeTower::box_coordinates`.
mod bottom;
mod cone;
pub(super) use cone::{ConeOptions, cone_base};
mod rounding;

use super::writer::Writer;
use crate::geometry::{
    ClipperError, CoordinateScale, Point, Polygon, Polyline, union_polygons_paths,
};

#[derive(Clone, Copy)]
pub(super) struct BoxCoordinates {
    pub(super) lower: [f32; 2],
    pub(super) upper: [f32; 2],
}
impl BoxCoordinates {
    pub(super) fn new(lower: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            lower,
            upper: [lower[0] + size[0], lower[1] + size[1]],
        }
    }
    pub(super) fn size(self) -> [f32; 2] {
        [self.upper[0] - self.lower[0], self.upper[1] - self.lower[1]]
    }
    pub(super) fn translate(&mut self, offset: [f32; 2]) {
        for (axis, value) in offset.into_iter().enumerate() {
            self.lower[axis] += value;
            self.upper[axis] += value;
        }
    }
    pub(super) fn corners(self) -> [[f32; 2]; 4] {
        [
            self.lower,
            [self.upper[0], self.lower[1]],
            self.upper,
            [self.lower[0], self.upper[1]],
        ]
    }
    pub(super) fn polygon(self, scale: CoordinateScale) -> Polygon {
        Polygon::new(self.corners().map(|p| scaled(p, scale)).to_vec())
    }
    fn polygon_new_scale(self, scale: CoordinateScale) -> Polygon {
        Polygon::new(self.corners().map(|p| new_scale(p, scale)).to_vec())
    }
}

pub(super) struct Shape {
    pub(super) width: f32,
    pub(super) depth: f32,
    pub(super) height: f32,
    pub(super) z: f32,
    pub(super) y_shift: f32,
    pub(super) rib_width: f32,
    pub(super) rib_length: f32,
    pub(super) fillet: bool,
}

impl Shape {
    pub(super) fn rib_polygon(
        &self,
        bounds: BoxCoordinates,
        scale: CoordinateScale,
    ) -> Result<Polygon, ClipperError> {
        let origin = Point::new(0, 0);
        let opposite = new_scale([self.width, self.depth], scale);
        let diagonal = (((opposite.x() as f64).powi(2) + (opposite.y() as f64).powi(2)).sqrt()
            * scale.factor()) as f32;
        let extra = (self.rib_length - diagonal).max(0.0) / 2.0;
        let extra = (((self.height - self.z).abs() / self.height * extra) / scale.factor() as f32)
            as i64 as f32;
        let shift = (self.y_shift / scale.factor() as f32) as i64;
        let half_width = (self.rib_width / scale.factor() as f32) as i64 / 2;
        let lines = [
            (origin, opposite),
            (Point::new(opposite.x(), 0), Point::new(0, opposite.y())),
        ];
        let mut polygons = lines
            .map(|(a, b)| {
                let dx = (b.x() - a.x()) as f64;
                let dy = (b.y() - a.y()) as f64;
                let length = (dx * dx + dy * dy).sqrt();
                let offset = [
                    (f64::from(extra) * (dx / length)) as i64,
                    (f64::from(extra) * (dy / length)) as i64,
                ];
                diagonal_rectangle(
                    Point::new(a.x() - offset[0], a.y() - offset[1] - shift),
                    Point::new(b.x() + offset[0], b.y() + offset[1] - shift),
                    half_width,
                )
            })
            .to_vec();
        polygons.push(bounds.polygon_new_scale(scale));
        Ok(union_polygons_paths(&polygons)?.remove(0))
    }

    pub(super) fn wall_polygon(
        &self,
        bounds: BoxCoordinates,
        rib: bool,
        scale: CoordinateScale,
    ) -> Result<Polygon, ClipperError> {
        let mut wall = if rib {
            self.rib_polygon(bounds, scale)?
        } else {
            bounds.polygon(scale)
        };
        if self.fillet && (rib || self.y_shift <= 1e-4) {
            if rib {
                wall = rounding::round_polygon(&wall, scale);
            }
            wall = union_polygons_paths(&[wall, bounds.polygon(scale)])?.remove(0);
        }
        Ok(wall)
    }
}

pub(super) fn emit_wall(
    writer: &mut Writer,
    polygon: &Polygon,
    feedrate: f32,
    retract: (f32, f32),
    scale: CoordinateScale,
) {
    // The source skip-points caller supplies an empty vector, so its gap helper
    // returns the original closed polygon (cpp 2530).
    let mut points = polygon.points().to_vec();
    points.push(points[0]);
    writer.generate_path(&[Polyline::new(points)], feedrate, retract, scale);
}

pub(super) fn scaled(point: [f32; 2], scale: CoordinateScale) -> Point {
    let factor = scale.factor() as f32;
    Point::new((point[0] / factor) as i64, (point[1] / factor) as i64)
}

// Point::new_scale uses the double-precision scale_ macro, unlike scaled(Vec2f).
pub(super) fn new_scale(point: [f32; 2], scale: CoordinateScale) -> Point {
    Point::new(
        (f64::from(point[0]) / scale.factor()) as i64,
        (f64::from(point[1]) / scale.factor()) as i64,
    )
}

pub(super) fn unscaled(point: Point, scale: CoordinateScale) -> [f32; 2] {
    [
        scale.unscale(point.x()) as f32,
        scale.unscale(point.y()) as f32,
    ]
}

fn diagonal_rectangle(a: Point, b: Point, offset: i64) -> Polygon {
    let dx = (b.x() - a.x()) as f64;
    let dy = (b.y() - a.y()) as f64;
    let length = (dx * dx + dy * dy).sqrt();
    let x = -dy / length * offset as f64;
    let y = dx / length * offset as f64;
    Polygon::new(vec![
        Point::new(
            (a.x() as f64 + x).round() as i64,
            (a.y() as f64 + y).round() as i64,
        ),
        Point::new(
            (a.x() as f64 - x).round() as i64,
            (a.y() as f64 - y).round() as i64,
        ),
        Point::new(
            (b.x() as f64 - x).round() as i64,
            (b.y() as f64 - y).round() as i64,
        ),
        Point::new(
            (b.x() as f64 + x).round() as i64,
            (b.y() as f64 + y).round() as i64,
        ),
    ])
}
