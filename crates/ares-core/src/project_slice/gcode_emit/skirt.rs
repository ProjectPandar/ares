//! Skirt generation and emission — rewrite slice of `Print.cpp:_make_skirt`
//! (2646-2990, single combined group) and `GCode.cpp::generate_skirt`
//! (4388-4451): convex hull of the first `skirt_height` layers, round-join
//! offsets at `skirt_distance` stepping by the flow spacing, loops reversed
//! to outermost-first (`Print.cpp:2985`), emitted per layer with the
//! `skirt_speed` override (`GCode.cpp:6599-6604`) and seam-gap clipped
//! loops split at the angle-derived start point
//! (`GCode.cpp:4334-4359`, `extrude_loop` seam gap).

use crate::geometry::{
    CoordinateScale, JoinType, Line, Point, Polygon, offset_paths, simplify_closed_points,
};
use crate::project_slice::gcode_emit::motion::{self, EmitState, LayerGeometry};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::project_slice::perimeters::flow::build_nonbridging_flow;
use crate::{FloatOrPercent, SliceError};

#[cfg(test)]
mod tests;

pub(super) struct SkirtPlan {
    /// Closed loops in emission order — outermost first (`Print.cpp:2985`).
    loops: Vec<Vec<Point>>,
    /// First-layer flow values; later layers rescale `mm3_per_mm` by the
    /// actual layer height (`GCode.cpp:4413`).
    width: f32,
    spacing: f32,
    /// Number of layers the skirt is printed on (`skirt_height`, or every
    /// layer when the draft shield makes it infinite).
    layer_count: usize,
    /// Seam target angle in degrees (`skirt_start_angle`).
    start_angle_deg: f64,
    single_loop_draft_shield: bool,
}

impl SkirtPlan {
    pub(super) fn generate(
        traversal: &PreparedPostClassicTraversal,
        brim: Option<&super::brim::BrimPlan>,
    ) -> Result<Option<Self>, SliceError> {
        let print = &traversal.resolved.views.full.process.print;
        let loops_count = usize::try_from(print.skirt_loops.0).unwrap_or(0);
        if loops_count == 0 {
            return Ok(None);
        }
        if print.skirt_type != crate::ProcessSkirtType::Combined {
            return Err(SliceError::UnsupportedProjectFeature(
                "skirt_type per-object".to_owned(),
            ));
        }
        let infinite_skirt = print.draft_shield == crate::ProcessDraftShield::Enabled;
        let layer_limit = if infinite_skirt {
            usize::MAX
        } else {
            usize::try_from(print.skirt_height.0).unwrap_or(0)
        };
        if layer_limit == 0 {
            return Ok(None);
        }

        // Occupied hull points: outer contours of the layers the skirt
        // covers (`Print.cpp:2670-2686`).
        let mut occupied = Vec::new();
        let mut total_layers = 0usize;
        for object in &traversal.objects {
            let layers = &object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .object
                .as_parts()
                .1;
            total_layers = total_layers.max(layers.len());
            for layer in layers.iter().take(layer_limit) {
                occupied.extend(
                    layer
                        .iter()
                        .flat_map(|expolygon| expolygon.contour().points().iter().copied()),
                );
            }
        }
        if let Some(brim) = brim {
            occupied.extend_from_slice(brim.covered_hull());
        }
        let hull = convex_hull(&occupied);
        if hull.len() < 3 {
            return Ok(None);
        }

        // `Print::skirt_flow` (`Print.cpp:2028-2049`).
        let full = &traversal.resolved.views.full;
        let object = traversal.resolved.objects.first();
        let initial_width = match print.initial_layer_line_width {
            FloatOrPercent::Float(value) if value <= 0.0 => object
                .map_or(full.process.object.line_width, |value| {
                    value.object.line_width
                }),
            value => value,
        };
        let nozzle = full
            .project
            .print
            .nozzle_diameter
            .0
            .first()
            .map_or(0.4, |value| value.0) as f32;
        let flow = build_nonbridging_flow(
            initial_width,
            print.initial_layer_print_height.0 as f32,
            nozzle,
        )?;

        // Loop centerlines (`Print.cpp:2718-2772`): start at
        // `skirt_distance - spacing/2`, step by spacing, inner-to-outer,
        // then reverse for emission.
        let scale = traversal.scale;
        let spacing = checked_scale(scale, flow.spacing as f64)?;
        let mut distance =
            checked_scale(scale, print.skirt_distance.0 - flow.spacing as f64 * 0.5)? + spacing;
        let arc_tolerance = checked_scale(scale, 0.1)? as f64;
        let hull_polygon = Polygon::new(hull);
        let minimum_filament = print.min_skirt_length.0;
        let filament = &full.filament.gcode;
        let diameter = filament
            .filament_diameter
            .0
            .first()
            .map_or(1.75, |diameter| diameter.0);
        let flow_ratio = super::motion::first_nullable_float(&filament.filament_flow_ratio, 1.0);
        let e_per_path_mm =
            flow.mm3_per_mm * flow_ratio / (std::f64::consts::PI * diameter.powi(2) * 0.25);
        let mut extruded_filament = 0.0;
        let mut loops = Vec::with_capacity(loops_count);
        while loops.len() < loops_count || extruded_filament < minimum_filament {
            let offset = offset_paths(
                std::slice::from_ref(&hull_polygon),
                (distance as f32).abs(),
                JoinType::Round,
                arc_tolerance,
            )
            .map_err(|_| SliceError::InvalidInput("skirt offset failed".to_owned()))?;
            let Some(loop_polygon) = offset.first() else {
                break;
            };
            let simplified = simplify_closed_points(
                loop_polygon.points().to_vec(),
                checked_scale(scale, 0.05)? as f64,
            );
            if simplified.len() < 3 {
                break;
            }
            extruded_filament += closed_length(&simplified) * scale.factor() * e_per_path_mm;
            loops.push(simplified);
            distance += spacing;
        }
        if loops.is_empty() {
            return Ok(None);
        }
        loops.reverse();

        let layer_count = if infinite_skirt {
            total_layers
        } else {
            layer_limit.min(total_layers)
        };
        let start_angle_deg = object.map_or(-135.0, |value| value.object.skirt_start_angle.0);
        Ok(Some(Self {
            loops,
            width: flow.width,
            spacing: flow.spacing,
            layer_count,
            start_angle_deg,
            single_loop_draft_shield: print.single_loop_draft_shield.0,
        }))
    }

    pub(super) fn covered_bounds(&self, scale: CoordinateScale) -> Option<(f64, f64, f64, f64)> {
        let mut bounds = None::<(f64, f64, f64, f64)>;
        for point in self.loops.iter().flatten() {
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

    /// Emits the loops for `layer_index`, split at the seam point
    /// (`GCode.cpp:4334-4359`, `GCode.cpp:4435-4441`).
    pub(super) fn emit(
        &self,
        output: &mut Vec<u8>,
        layer: SkirtLayer,
        geometry: LayerGeometry<'_>,
        state: &mut EmitState,
    ) {
        if layer.index >= self.layer_count {
            return;
        }
        let height = layer.height_mm as f32;
        let rounded_rectangle = 1.0 - 0.25 * std::f64::consts::PI;
        let mm3_per_mm = f64::from(
            (layer.height_mm * (f64::from(self.width) - layer.height_mm * rounded_rectangle))
                as f32,
        );
        let flow = motion::SkirtLoopFlow {
            width: self.width,
            height,
            mm3_per_mm,
        };
        let loops = self.loops_for_layer(layer.index);
        let mut seam_target = if layer.index == 0 {
            find_start_point(&loops[0], self.start_angle_deg)
        } else {
            state.last_scaled_position.map_or_else(
                || find_start_point(&loops[0], self.start_angle_deg),
                |(x, y)| Point::new(x, y),
            )
        };
        for loop_points in loops {
            let split = split_at_nearest(loop_points, seam_target);
            motion::emit_skirt_loop(
                output,
                split.into_iter().map(|point| (point.x(), point.y())),
                flow,
                geometry,
                state,
            );
            if let Some(last) = state.last_scaled_position {
                seam_target = Point::new(last.0, last.1);
            }
        }
    }

    fn loops_for_layer(&self, layer_index: usize) -> &[Vec<Point>] {
        if layer_index > 0 && self.single_loop_draft_shield {
            &self.loops[self.loops.len() - 1..]
        } else {
            &self.loops
        }
    }
}

/// Per-layer emission inputs.
pub(super) struct SkirtLayer {
    pub(super) index: usize,
    pub(super) height_mm: f64,
}

fn closed_length(points: &[Point]) -> f64 {
    (0..points.len())
        .map(|index| Line::new(points[index], points[(index + 1) % points.len()]).length())
        .sum()
}

/// Convex hull in counter-clockwise order — rewrite of
/// `Geometry/ConvexHull.cpp:11-43` with the `Geometry.hpp:36` orientation
/// predicate.
pub(in crate::project_slice::gcode_emit) fn convex_hull(points: &[Point]) -> Vec<Point> {
    let mut sorted = points.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let count = sorted.len();
    if count < 3 {
        return Vec::new();
    }
    let ccw = |a: Point, b: Point, c: Point| {
        let cross = (i128::from(b.x()) - i128::from(a.x()))
            * (i128::from(c.y()) - i128::from(a.y()))
            - (i128::from(b.y()) - i128::from(a.y())) * (i128::from(c.x()) - i128::from(a.x()));
        cross > 0
    };
    let mut hull = Vec::with_capacity(2 * count);
    for point in &sorted {
        while hull.len() >= 2 && !ccw(hull[hull.len() - 2], hull[hull.len() - 1], *point) {
            hull.pop();
        }
        hull.push(*point);
    }
    let lower_end = hull.len() + 1;
    for point in sorted[..count - 1].iter().rev() {
        while hull.len() >= lower_end && !ccw(hull[hull.len() - 2], hull[hull.len() - 1], *point) {
            hull.pop();
        }
        hull.push(*point);
    }
    hull.pop();
    hull
}

/// Seam target on the first loop's bounding-circle radius at the configured
/// angle (`GCode.cpp:4334-4359`).
fn find_start_point(points: &[Point], start_angle_deg: f64) -> Point {
    let mut min_x = i64::MAX;
    let mut max_x = i64::MIN;
    let mut min_y = i64::MAX;
    let mut max_y = i64::MIN;
    for point in points {
        let x = point.x();
        let y = point.y();
        if x < min_x {
            min_x = x;
        } else if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        } else if y > max_y {
            max_y = y;
        }
    }
    let center_x = (min_x + max_x) as f64 / 2.0;
    let center_y = (min_y + max_y) as f64 / 2.0;
    let radius = ((center_x - min_x as f64).powi(2) + (center_y - min_y as f64).powi(2)).sqrt();
    let radians = start_angle_deg.to_radians();
    Point::new(
        (center_x + radius * radians.cos()) as i64,
        (center_y + radius * radians.sin()) as i64,
    )
}

/// Splits the closed loop at the point nearest `target` — rewrite of
/// `ExtrusionLoop::split_at` (`ExtrusionEntity.cpp:261`) over the open
/// polyline form (`Print.cpp:2745` split_at_first_point): per-segment
/// projections with endpoint clamping (`Point.cpp:106-129`), first segment
/// wins ties, the loop rotates to the seam and repeats it.
fn split_at_nearest(points: &[Point], target: Point) -> Vec<Point> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let mut best_distance = f64::MAX;
    let mut seam = points[0];
    let mut seam_index = 0usize;
    for (index, pair) in points.windows(2).enumerate() {
        let foot = projection_onto(pair[0], pair[1], target);
        let dx = (foot.x() - target.x()) as f64;
        let dy = (foot.y() - target.y()) as f64;
        let distance = dx * dx + dy * dy;
        if distance < best_distance {
            best_distance = distance;
            seam = foot;
            seam_index = index;
        }
    }
    // Note: upstream additionally snaps to a segment endpoint when the
    // target is within 0.001mm of it (`ExtrusionEntity.cpp:268-280`); the
    // clamped projection above already lands exactly on endpoints there.
    let mut out = Vec::with_capacity(points.len() + 1);
    if seam == points[seam_index + 1] {
        out.extend_from_slice(&points[seam_index + 1..]);
        out.extend_from_slice(&points[..=seam_index]);
        out.push(seam);
    } else {
        out.push(seam);
        out.extend_from_slice(&points[seam_index + 1..]);
        out.extend_from_slice(&points[..=seam_index]);
        out.push(seam);
    }
    out
}

/// `Point::projection_onto` (`Point.cpp:106-129`): affine projection onto
/// the segment, clamped to the nearest endpoint, truncated to integer
/// coordinates.
fn projection_onto(a: Point, b: Point, point: Point) -> Point {
    if a == b {
        return a;
    }
    let lx = (b.x() - a.x()) as f64;
    let ly = (b.y() - a.y()) as f64;
    let theta =
        ((b.x() - point.x()) as f64 * lx + (b.y() - point.y()) as f64 * ly) / (lx * lx + ly * ly);
    if (0.0..=1.0).contains(&theta) {
        return Point::new(
            (theta * a.x() as f64 + (1.0 - theta) * b.x() as f64) as i64,
            (theta * a.y() as f64 + (1.0 - theta) * b.y() as f64) as i64,
        );
    }
    let da = (a.x() - point.x()).pow(2) + (a.y() - point.y()).pow(2);
    let db = (b.x() - point.x()).pow(2) + (b.y() - point.y()).pow(2);
    if da < db { a } else { b }
}

fn checked_scale(scale: CoordinateScale, millimeters: f64) -> Result<i64, SliceError> {
    scale.checked_scale(millimeters).ok_or_else(|| {
        SliceError::InvalidInput("skirt geometry is outside the supported range".to_owned())
    })
}
