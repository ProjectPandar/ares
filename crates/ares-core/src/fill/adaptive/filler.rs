//! OrcaSlicer 2.4.2 `Fill/FillAdaptive.cpp` infill line generation
//! (`FillContext`, `generate_infill_lines_recursive`,
//! `Filler::_fill_surface_single`; lines 232-235, 362-399, 450-510,
//! 1321-1428).
//!
//! Slice 2 of the FillAdaptive port. The octree cube centers are in world
//! coordinates (the build ends with `transform_center`,
//! `FillAdaptive.cpp:1524-1528`); `z` and the surface live in the same
//! frame.

use super::octree::{Cube, Octree};
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polyline};

/// `FillAdaptive.cpp:232-235` — traversal order of octree children cells
/// for the three infill directions.
const CHILD_TRAVERSAL_ORDER: [[usize; 8]; 3] = [
    [2, 3, 0, 1, 6, 7, 4, 5],
    [4, 0, 6, 2, 5, 1, 7, 3],
    [1, 5, 0, 4, 3, 7, 2, 6],
];

/// `FillAdaptive.cpp:365-367` — the angles have to agree with the
/// traversal order above.
const DIRECTION_ANGLES: [f64; 3] = [
    0.0,
    (2.0 * std::f64::consts::PI) / 3.0,
    -(2.0 * std::f64::consts::PI) / 3.0,
];

/// A generated infill segment in scaled coordinates.
#[derive(Clone, Copy, Debug)]
struct Segment {
    a: Point,
    b: Point,
}

/// `FillAdaptive.cpp:362-399`
struct FillContext<'a> {
    cubes_properties: &'a [super::octree::CubeProperties],
    z_position: f64,
    traversal_order: [usize; 8],
    cos_a: f64,
    sin_a: f64,
    /// Linearized tree spanning a single octree wall; `None` marks an
    /// unused slot (upstream uses `a.x == coord_t::max`).
    temp_lines: Vec<Option<Segment>>,
    output_lines: Vec<Segment>,
    /// Gap threshold for starting a new line, in scaled units
    /// (`FillAdaptive.cpp:489`: 1000 at the 40/mm lattice).
    connect_epsilon: i64,
    scale: CoordinateScale,
}

impl<'a> FillContext<'a> {
    fn new(
        octree: &'a Octree,
        z_position: f64,
        direction_idx: usize,
        connect_epsilon: i64,
        scale: CoordinateScale,
    ) -> Self {
        let angle = DIRECTION_ANGLES[direction_idx];
        FillContext {
            cubes_properties: &octree.cubes_properties,
            z_position,
            traversal_order: CHILD_TRAVERSAL_ORDER[direction_idx],
            cos_a: angle.cos(),
            sin_a: angle.sin(),
            temp_lines: vec![None; (1usize << octree.cubes_properties.len()) - 1],
            output_lines: Vec::new(),
            connect_epsilon,
            scale,
        }
    }

    /// `FillAdaptive.cpp:388` — same convention as `Point::rotate()`.
    fn rotate(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.cos_a * x - self.sin_a * y,
            self.sin_a * x + self.cos_a * y,
        )
    }
}

/// `FillAdaptive.cpp:450-510` — discretize the walls splitting each cube;
/// touching lines of the same direction merge through `temp_lines`.
fn generate_infill_lines_recursive(
    context: &mut FillContext,
    cube: &Cube,
    address: usize,
    depth: i32,
) {
    let properties = &context.cubes_properties[depth as usize];
    let z_diff = context.z_position - cube.center()[2];
    let z_diff_abs = z_diff.abs();

    if z_diff_abs > properties.height / 2.0 {
        return;
    }

    if z_diff_abs < properties.line_z_distance {
        // Discretize a single wall splitting the cube into two.
        let zdist = properties.line_z_distance;
        let from_x = 0.5 * properties.diagonal_length * (zdist - z_diff_abs) / zdist;
        let from_y = properties.line_xy_distance - (zdist + z_diff) / 2.0f64.sqrt();
        let (rot_from_x, rot_from_y) = context.rotate(from_x, from_y);
        let (rot_to_x, rot_to_y) = context.rotate(-from_x, from_y);
        let offset = (cube.center()[0], cube.center()[1]);
        let from = (rot_from_x + offset.0, rot_from_y + offset.1);
        let to = (rot_to_x + offset.0, rot_to_y + offset.1);
        // Either extend an existing line or start a new one.
        let new_a = Point::new(
            context
                .scale
                .checked_scale_rounded(from.0)
                .unwrap_or(i64::MAX),
            context
                .scale
                .checked_scale_rounded(from.1)
                .unwrap_or(i64::MAX),
        );
        let new_b = Point::new(
            context
                .scale
                .checked_scale_rounded(to.0)
                .unwrap_or(i64::MAX),
            context
                .scale
                .checked_scale_rounded(to.1)
                .unwrap_or(i64::MAX),
        );
        let slot = &mut context.temp_lines[address];
        match *slot {
            None => *slot = Some(Segment { a: new_a, b: new_b }),
            Some(ref mut last) => {
                let gap = (new_a.x() - last.b.x())
                    .abs()
                    .max((new_a.y() - last.b.y()).abs());
                if gap > context.connect_epsilon {
                    let finished = *last;
                    context.output_lines.push(finished);
                    last.a = new_a;
                }
                last.b = new_b;
            }
        }
    }

    // left child index
    let mut address = address * 2 + 1;
    let depth = depth - 1;
    let mut i = 0;
    let order = context.traversal_order;
    for &child_idx in &order {
        if let Some(child) = cube.child(child_idx) {
            generate_infill_lines_recursive(context, child, address, depth);
        }
        i += 1;
        if i == 4 {
            // right child index
            address += 1;
        }
    }
}

/// `Filler::_fill_surface_single` (`FillAdaptive.cpp:1321-1428`): generate
/// the three line directions, crop by the surface, then chain/connect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fill_surface(
    octree: &Octree,
    surface: &ExPolygon,
    z: f64,
    spacing: f64,
    multiline: i32,
    anchor_length: f32,
    anchor_length_max: f32,
    dont_sort: bool,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // `FillAdaptive.cpp:489`: 1000 at the 40/mm lattice == 25 mm.
    let connect_epsilon = (25.0 / scale.factor()) as i64;
    let mut lines: Vec<Segment> = Vec::new();
    for direction in 0..3 {
        let mut context = FillContext::new(octree, z, direction, connect_epsilon, scale);
        generate_infill_lines_recursive(
            &mut context,
            octree.root(),
            0,
            octree.cubes_properties.len() as i32 - 1,
        );
        lines.append(&mut context.output_lines);
        for line in context.temp_lines.into_iter().flatten() {
            lines.push(line);
        }
    }
    let mut all_polylines: Vec<Polyline> = lines
        .into_iter()
        .map(|l| Polyline::new(vec![l.a, l.b]))
        .collect();

    if let Ok(path) = std::env::var("ARES_DUMP_ADPLINES") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "RAW z={z:.3} n={}", all_polylines.len());
            for polyline in &all_polylines {
                let pts = polyline.points();
                if let (Some(a), Some(b)) = (pts.first(), pts.last()) {
                    let _ = writeln!(
                        file,
                        "L {:+.4} {:+.4} -> {:+.4} {:+.4}",
                        a.x() as f64 * scale.factor(),
                        a.y() as f64 * scale.factor(),
                        b.x() as f64 * scale.factor(),
                        b.y() as f64 * scale.factor()
                    );
                }
            }
        }
    }

    // Apply multiline offset if needed (`FillAdaptive.cpp:1380`).
    all_polylines =
        super::super::multiline_offset::apply(all_polylines, multiline, spacing, scale)?;

    // Crop all polylines (`FillAdaptive.cpp:1383`).
    let clip = expolygon_polygons(surface);
    all_polylines = crate::geometry::intersection_open_polylines(&all_polylines, &clip)?;

    if let Ok(path) = std::env::var("ARES_DUMP_ADPLINES") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "CROP z={z:.3} n={}", all_polylines.len());
            for polyline in &all_polylines {
                let pts = polyline.points();
                if let (Some(a), Some(b)) = (pts.first(), pts.last()) {
                    let _ = writeln!(
                        file,
                        "C {:+.4} {:+.4} -> {:+.4} {:+.4} pts={}",
                        a.x() as f64 * scale.factor(),
                        a.y() as f64 * scale.factor(),
                        b.x() as f64 * scale.factor(),
                        b.y() as f64 * scale.factor(),
                        pts.len()
                    );
                }
            }
        }
    }

    if all_polylines.len() <= 1 {
        return Ok(all_polylines);
    }

    if multiline == 1 {
        // After the intersection some polylines with only one line are
        // split into more (`FillAdaptive.cpp:1388-1394`).
        for polyline in &mut all_polylines {
            let points = polyline.points();
            if points.len() > 2 {
                *polyline = Polyline::new(vec![*points.first().unwrap(), *points.last().unwrap()]);
            }
        }
        // `FillAdaptive.cpp:1399-1401`: hooks before the chain/connect
        // tail (in scaled units, matching the polylines' lattice).
        if all_polylines.len() > 1 {
            let scaled_spacing = spacing / scale.factor();
            let hook_length = (f64::from(anchor_length) / scale.factor()).min(i64::MAX as f64);
            let hook_length_max =
                (f64::from(anchor_length_max) / scale.factor()).min(i64::MAX as f64);
            all_polylines = crate::fill::adaptive::hooks::connect_lines_using_hooks(
                all_polylines,
                surface,
                scaled_spacing,
                hook_length,
                hook_length_max,
                1.0 / scale.factor(),
            );
        }
    }

    super::super::connect::connect_infill(
        all_polylines,
        surface,
        spacing,
        super::super::connect::FillConnectionParams {
            anchor_length,
            anchor_length_max,
            multiline,
            dont_sort,
        },
        scale,
    )
}

#[cfg(test)]
mod tests;

fn expolygon_polygons(expolygon: &ExPolygon) -> Vec<crate::geometry::Polygon> {
    let mut polygons = vec![expolygon.contour().clone()];
    polygons.extend(expolygon.holes().iter().cloned());
    polygons
}
