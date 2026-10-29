//! PRISM kernel and planar consumers of `Feature/Interlocking/VoxelUtils`.

use std::collections::BTreeSet;

use crate::{
    fill::rectilinear::{MonotonicFillParams, fill_rectilinear_surface},
    geometry::{BoundingBox, CoordinateScale, ExPolygon, Point, Polygon},
};

use super::GeometryResult;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct GridPoint3 {
    pub(super) x: i64,
    pub(super) y: i64,
    pub(super) z: i64,
}

pub(super) struct DilationKernel {
    size: i64,
    relative_cells: Vec<GridPoint3>,
}

impl DilationKernel {
    pub(super) fn prism(size: i64) -> Self {
        let relative_cells = prism_cells(size);
        Self {
            size,
            relative_cells,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct VoxelUtils {
    pub(super) cell_width: i64,
    pub(super) cell_layers: i64,
    scale: CoordinateScale,
}

impl VoxelUtils {
    pub(super) fn new(cell_width: i64, cell_layers: i64, scale: CoordinateScale) -> Self {
        Self {
            cell_width,
            cell_layers,
            scale,
        }
    }

    fn grid_point(self, point: Point, z: i64) -> GridPoint3 {
        GridPoint3 {
            x: grid_coord(point.x(), self.cell_width),
            y: grid_coord(point.y(), self.cell_width),
            z: grid_coord(z, self.cell_layers),
        }
    }

    pub(super) fn lower_corner(self, point: GridPoint3) -> (Point, i64) {
        (
            Point::new(point.x * self.cell_width, point.y * self.cell_width),
            point.z * self.cell_layers,
        )
    }

    pub(super) fn polygon(self, point: GridPoint3) -> ExPolygon {
        let (lower, _) = self.lower_corner(point);
        let x = lower.x();
        let y = lower.y();
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(x, y),
                Point::new(x + self.cell_width, y),
                Point::new(x + self.cell_width, y + self.cell_width),
                Point::new(x, y + self.cell_width),
            ]),
            Vec::new(),
        )
    }

    fn walk_line(self, from: Point, to: Point, z: i64, output: &mut BTreeSet<GridPoint3>) {
        let mut current = self.grid_point(from, z);
        output.insert(current);
        if current == self.grid_point(to, z) {
            return;
        }
        let dx = to.x() - from.x();
        let dy = to.y() - from.y();
        loop {
            let px = crossing(current.x, from.x(), dx, self.cell_width);
            let py = crossing(current.y, from.y(), dy, self.cell_width);
            if px.min(py) > 1.0 {
                return;
            }
            if px <= py {
                current.x += dx.signum();
            } else {
                current.y += dy.signum();
            }
            output.insert(current);
        }
    }

    fn walk_polygon(self, polygon: &Polygon, z: i64, output: &mut BTreeSet<GridPoint3>) {
        let points = polygon.points();
        let mut previous = points[points.len() - 1];
        for &point in points {
            self.walk_line(previous, point, z, output);
            previous = point;
        }
    }

    pub(super) fn walk_dilated_polygons(
        self,
        polygons: &[ExPolygon],
        z: i64,
        kernel: &DilationKernel,
        output: &mut BTreeSet<GridPoint3>,
    ) {
        let delta = (1 - kernel.size % 2) * self.cell_width / 2;
        let dz = (1 - kernel.size % 2) * self.cell_layers / 2;
        let mut boundary = BTreeSet::new();
        for polygon in polygons {
            let translated = translate(polygon, delta);
            self.walk_polygon(translated.contour(), z + dz, &mut boundary);
            for hole in translated.holes() {
                self.walk_polygon(hole, z + dz, &mut boundary);
            }
        }
        for point in boundary {
            emplace(point, kernel, output);
        }
    }

    pub(super) fn walk_dilated_areas(
        self,
        polygon: &ExPolygon,
        z: i64,
        kernel: &DilationKernel,
        output: &mut BTreeSet<GridPoint3>,
    ) -> GeometryResult<()> {
        let delta = (1 - kernel.size % 2) * self.cell_width / 2 - self.cell_width / 2;
        let dz = (1 - kernel.size % 2) * self.cell_layers / 2 - self.cell_layers / 2;
        let translated = translate(polygon, delta);
        self.walk_areas_inner(&translated, z + dz, kernel, output)
    }

    fn walk_areas_inner(
        self,
        polygon: &ExPolygon,
        z: i64,
        kernel: &DilationKernel,
        output: &mut BTreeSet<GridPoint3>,
    ) -> GeometryResult<()> {
        let params = MonotonicFillParams {
            spacing: self.cell_width as f64 * self.scale.factor(),
            overlap: 0.0,
            density: 1.0,
            angle: std::f32::consts::FRAC_PI_2,
            layer_index: 0,
            thickness_layers: 1,
            fixed_angle: true,
            // `spreadDotsArea` builds `Fill::new_from_type(ipAlignedRectilinear)`
            // (`VoxelUtils.cpp:145`) and never sets `layer_id`, so
            // `_infill_direction` skips the `_layer_angle` alternation on both
            // counts (`FillRectilinear.hpp:44-46`, `FillBase.cpp:300-316`).
            layer_angle_fixed: true,
            dont_alternate_fill_direction: true,
            bridge_angle: None,
            reference_point: BoundingBox::from_expolygon(polygon).unwrap().center(),
            dont_adjust: true,
            anchor_length_max: 0.0,
            link_max_length: 0.0,
            consistent_pattern: false,
            horiz_move: 0.0,
            extended_object_min_x: None,
            extended_object_max_x: None,
            locked_zag: false,
        };
        let lines = fill_rectilinear_surface(polygon, params, self.scale)?;
        for line in lines.polylines {
            let points = line.points();
            let (mut a, mut b) = (points[0], points[1]);
            if a.y() > b.y() {
                std::mem::swap(&mut a, &mut b);
            }
            let start = a.y() - a.y() % self.cell_width - self.cell_width;
            for y in (start..b.y())
                .step_by(self.cell_width as usize)
                .filter(|&y| y >= a.y())
            {
                let point = Point::new(a.x() + self.cell_width / 2, y + self.cell_width / 2);
                emplace(self.grid_point(point, z), kernel, output);
            }
        }
        Ok(())
    }
}

fn prism_cells(size: i64) -> Vec<GridPoint3> {
    let half = size / 2;
    let start = -half;
    let end = size - half;
    let mult = size * size * size;
    let mut relative_cells = Vec::new();
    for x in start..end {
        let limit_x = if x < 0 { start } else { end - 1 };
        let limit_x = if limit_x == 0 { 1 } else { limit_x };
        for y in start..end {
            let limit_y = if y < 0 { start } else { end - 1 };
            let limit_y = if limit_y == 0 { 1 } else { limit_y };
            if mult * x / limit_x + mult * y / limit_y <= mult {
                relative_cells.extend((start..end).map(|z| GridPoint3 { x, y, z }));
            }
        }
    }
    relative_cells
}

fn grid_coord(value: i64, cell: i64) -> i64 {
    value / cell - i64::from(value < 0)
}

fn crossing(current: i64, start: i64, diff: i64, cell: i64) -> f64 {
    if diff == 0 {
        f64::MAX
    } else {
        let boundary = (current + i64::from(diff > 0)) * cell;
        (boundary - start) as f64 / diff as f64
    }
}

fn emplace(point: GridPoint3, kernel: &DilationKernel, output: &mut BTreeSet<GridPoint3>) {
    for relative in &kernel.relative_cells {
        let point = GridPoint3 {
            x: point.x + relative.x,
            y: point.y + relative.y,
            z: point.z + relative.z,
        };
        if point.z >= 0 {
            output.insert(point);
        }
    }
}

fn translate(polygon: &ExPolygon, delta: i64) -> ExPolygon {
    let translated = |ring: &Polygon| {
        Polygon::new(
            ring.points()
                .iter()
                .map(|point| Point::new(point.x() + delta, point.y() + delta))
                .collect(),
        )
    };
    ExPolygon::new(
        translated(polygon.contour()),
        polygon.holes().iter().map(translated).collect(),
    )
}
