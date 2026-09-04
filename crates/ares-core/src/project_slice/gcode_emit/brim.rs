//! Explicit outer brim for a single object, rewriting the simple
//! `outer_inner_brim_area` + `makeBrimInfillImpl` path
//! (`Brim.cpp:421-570,819-878`).

use crate::{
    ProcessBrimType, SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, Point, Polygon, difference_ex, offset_expolygons,
        simplify_closed_points,
    },
    project_slice::{
        gcode_emit::motion::{self, EmitState, LayerGeometry},
        perimeters::{
            classic::traversal::PreparedPostClassicTraversal, flow::build_nonbridging_flow,
        },
    },
};

pub(super) struct BrimPlan {
    paths: Vec<Vec<Point>>,
    covered_hull: Vec<Point>,
    width: f32,
    spacing: f32,
    height: f32,
    mm3_per_mm: f64,
}

impl BrimPlan {
    pub(super) fn generate(
        traversal: &PreparedPostClassicTraversal,
    ) -> Result<Option<Self>, SliceError> {
        let Some(resolved) = traversal.resolved.objects.first() else {
            return Ok(None);
        };
        let options = &resolved.object;
        if options.brim_type != ProcessBrimType::OuterOnly || options.brim_width.0 <= 0.0 {
            return Ok(None);
        }
        let source = traversal
            .objects
            .first()
            .and_then(|object| {
                object
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object
                    .object
                    .as_parts()
                    .1
                    .first()
            })
            .cloned()
            .unwrap_or_default();
        if source.is_empty() {
            return Ok(None);
        }
        let full = &traversal.resolved.views.full;
        let print = &full.process.print;
        let region = &resolved
            .layer_candidates
            .first()
            .and_then(|layer| layer.model_parts.first())
            .expect("brim object has a resolved model-part region")
            .region;
        let width = if print.initial_layer_line_width.is_non_positive() {
            if region.inner_wall_line_width.is_non_positive() {
                options.line_width
            } else {
                region.inner_wall_line_width
            }
        } else {
            print.initial_layer_line_width
        };
        let nozzle = full
            .project
            .print
            .nozzle_diameter
            .0
            .first()
            .map_or(0.4, |diameter| diameter.0) as f32;
        let flow =
            build_nonbridging_flow(width, print.initial_layer_print_height.0 as f32, nozzle)?;
        let scale = traversal.scale;
        let spacing_mm = f64::from(flow.spacing);
        let brim_width_mm = (options.brim_width.0 / spacing_mm / 2.0).floor() * spacing_mm * 2.0;
        if brim_width_mm <= 0.0 {
            return Ok(None);
        }
        let gap = scaled_f32(scale, options.brim_object_gap.0)?;
        let brim_width = scaled_f32(scale, brim_width_mm)?;
        let spacing = scaled_f32(scale, spacing_mm)?;
        let resolution = scaled_f64(scale, 0.0125)?;
        let inner = offset_expolygons(&source, gap, JoinType::Round, resolution)
            .map_err(brim_geometry_error)?;
        let outer = offset_expolygons(&inner, brim_width, JoinType::Round, resolution)
            .map_err(brim_geometry_error)?;
        let covered_hull = super::skirt::convex_hull(
            &outer
                .iter()
                .flat_map(|expolygon| expolygon.contour().points().iter().copied())
                .collect::<Vec<_>>(),
        );
        let brim_area = difference_ex(&outer, &inner).map_err(brim_geometry_error)?;
        // `Brim.cpp:824-832` — every stage of the loop stepping runs a
        // douglas_peucker pass with `resolution` (0.0125 mm) between the
        // offsets: on the input area, after the −0.5 spacing opening, and
        // between the −1.3/+0.3 closing pair.
        let mut brim_area = brim_area;
        for expolygon in &mut brim_area {
            expolygon.douglas_peucker(resolution);
        }
        let mut area = offset_expolygons(&brim_area, -0.5 * spacing, JoinType::Round, resolution)
            .map_err(brim_geometry_error)?;
        for expolygon in &mut area {
            expolygon.douglas_peucker(resolution);
        }
        let mut loops = Vec::new();
        while !area.is_empty() {
            append_loops(&mut loops, &area, resolution);
            area = offset_expolygons(&area, -1.3 * spacing, JoinType::Round, resolution)
                .and_then(|mut area| {
                    for expolygon in &mut area {
                        expolygon.douglas_peucker(resolution);
                    }
                    offset_expolygons(&area, 0.3 * spacing, JoinType::Round, resolution)
                })
                .map_err(brim_geometry_error)?;
        }
        loops.sort_by(|left, right| polygon_area(right).total_cmp(&polygon_area(left)));
        let paths = loops
            .into_iter()
            .map(|mut points| {
                points.push(points[0]);
                points
            })
            .collect::<Vec<_>>();
        if paths.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            paths,
            covered_hull,
            width: flow.width,
            spacing: flow.spacing,
            height: flow.height,
            mm3_per_mm: flow.mm3_per_mm * options.brim_flow_ratio.0,
        }))
    }

    pub(super) fn covered_hull(&self) -> &[Point] {
        &self.covered_hull
    }

    pub(super) fn covered_bounds(&self, scale: CoordinateScale) -> Option<(f64, f64, f64, f64)> {
        let mut bounds = None::<(f64, f64, f64, f64)>;
        for point in self.paths.iter().flatten() {
            let x = scale.unscale(point.x());
            let y = scale.unscale(point.y());
            bounds = Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                }
                None => (x, y, x, y),
            });
        }
        let padding = 0.5 * f64::from(self.spacing);
        bounds.map(|(min_x, min_y, max_x, max_y)| {
            (
                min_x - padding,
                min_y - padding,
                max_x + padding,
                max_y + padding,
            )
        })
    }

    pub(super) fn emit(
        &self,
        output: &mut Vec<u8>,
        geometry: LayerGeometry<'_>,
        state: &mut EmitState,
    ) {
        for path in &self.paths {
            motion::emit_brim_loop(
                output,
                path.iter().map(|point| (point.x(), point.y())),
                motion::SkirtLoopFlow {
                    width: self.width,
                    height: self.height,
                    mm3_per_mm: self.mm3_per_mm,
                },
                geometry,
                state,
            );
        }
    }
}

fn append_loops(output: &mut Vec<Vec<Point>>, area: &[ExPolygon], tolerance: f64) {
    for expolygon in area {
        output.push(simplify_closed_points(
            expolygon.contour().points().to_vec(),
            tolerance,
        ));
        output.extend(
            expolygon
                .holes()
                .iter()
                .map(|hole| simplify_closed_points(hole.points().to_vec(), tolerance)),
        );
    }
    output.retain(|points| points.len() >= 3);
}

fn polygon_area(points: &[Point]) -> f64 {
    Polygon::new(points.to_vec()).area().abs()
}

fn scaled_f32(scale: CoordinateScale, value: f64) -> Result<f32, SliceError> {
    scale
        .checked_scale(value)
        .map(|value| value as f32)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".to_owned()))
}

fn scaled_f64(scale: CoordinateScale, value: f64) -> Result<f64, SliceError> {
    scale
        .checked_scale(value)
        .map(|value| value as f64)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".to_owned()))
}

fn brim_geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".to_owned())
}

#[cfg(test)]
mod tests;
