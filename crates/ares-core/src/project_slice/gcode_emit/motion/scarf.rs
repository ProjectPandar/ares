//! `GCode.cpp::extrude_loop` / `ExtrusionEntity.cpp::ExtrusionLoopSloped` scarf paths.

mod emit;
#[cfg(test)]
mod tests;

pub(super) use emit::segments as emit_segments;

use super::{LayerGeometry, MotionOptions};
use crate::project_slice::perimeters::classic::{
    chained_loops::ExtrusionLoopRole,
    materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
};
use crate::{FloatOrPercent, ProcessSeamScarfType};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Slope {
    pub(super) z_begin: f64,
    pub(super) z_end: f64,
    pub(super) e_begin: f64,
    pub(super) e_end: f64,
    pub(super) speed: f64,
    pub(super) flow_ratio: f64,
}

#[derive(Debug, PartialEq)]
pub(super) struct Path {
    pub(super) path: ExtrusionPath,
    pub(super) slope: Option<Slope>,
}

pub(super) struct Loop {
    pub(super) paths: Vec<Path>,
    pub(super) wipe_paths: Vec<ExtrusionPath>,
}

pub(super) fn build(
    original: &[ExtrusionPath],
    role: ExtrusionLoopRole,
    geometry: LayerGeometry<'_>,
    options: &MotionOptions,
    layer_index: usize,
) -> Option<Loop> {
    let first = original.first()?;
    if layer_index == 0
        || options.scarf.seam_slope_type == ProcessSeamScarfType::None
        || options.scarf.min_length <= 0.0
        || options.scarf.steps == 0
        || (options.scarf.seam_slope_type == ProcessSeamScarfType::External
            && role == ExtrusionLoopRole::Hole)
        || !matches!(
            first.role,
            ExtrusionRole::ExternalPerimeter | ExtrusionRole::Perimeter
        )
        || (first.role == ExtrusionRole::Perimeter && !options.scarf.inner_walls)
        || options.scarf.conditional
    {
        return None;
    }

    let scale = geometry.scale.factor();
    let loop_length = original
        .iter()
        .map(|path| path_length(&path.polyline.points) * scale)
        .sum::<f64>();
    let slope_length = if options.scarf.entire_loop {
        loop_length
    } else {
        options.scarf.min_length.min(loop_length)
    };
    if slope_length <= 0.0 {
        return None;
    }
    let start_ratio = match options
        .scarf
        .start_height
        .unwrap_or(FloatOrPercent::Float(0.0))
    {
        FloatOrPercent::Float(value) => value / f64::from(first.height),
        FloatOrPercent::Percent(value) => value.0 / 100.0,
    };
    let start_ratio = if start_ratio >= 1.0 {
        0.99
    } else {
        start_ratio
    };
    let speed = scarf_speed(first.role, loop_length, options);
    let max_segment = slope_length / options.scarf.steps as f64 / scale;
    let seam_gap = options.seam_gap / scale;
    let mut starts = Vec::new();
    let mut flats = Vec::new();
    let mut ends = Vec::new();
    let mut remaining = slope_length / scale;
    let total_slope = remaining;
    let mut ratio = start_ratio;
    let mut index = 0;

    while index < original.len() && remaining > 0.0 {
        let source = &original[index];
        let length = path_length(&source.polyline.points);
        let (slope_path, flat_path, consumed) = if length > remaining {
            let (slope, flat) = split_path(source, remaining);
            (slope, Some(flat), remaining)
        } else {
            (source.clone(), None, length)
        };
        remaining -= consumed;
        let end_ratio = 1.0_f64 + (start_ratio - 1.0) * (remaining / total_slope);
        add_slope(
            slope_path,
            ratio,
            end_ratio,
            max_segment,
            seam_gap,
            speed,
            options.scarf.flow_ratio,
            &mut starts,
            &mut ends,
        );
        ratio = end_ratio;
        if let Some(flat) = flat_path {
            flats.push(flat);
            index += 1;
            break;
        }
        index += 1;
    }
    flats.extend(original[index..].iter().cloned());

    let front_clip = if first.role == ExtrusionRole::Perimeter {
        starts
            .iter()
            .map(|path| path_length(&path.path.polyline.points))
            .sum::<f64>()
            * 0.4
    } else {
        seam_gap * 2.0
    };
    clip_front_paths(&mut starts, front_clip);
    clip_end_paths(&mut ends, seam_gap);

    let mut paths = Vec::with_capacity(starts.len() + flats.len() + ends.len());
    paths.extend(starts);
    paths.extend(flats.iter().cloned().map(|path| Path { path, slope: None }));
    paths.extend(ends.iter().map(|path| Path {
        path: path.path.clone(),
        slope: path.slope,
    }));
    let wipe_paths = if ends.is_empty() {
        original.to_vec()
    } else {
        flats
            .into_iter()
            .chain(ends.into_iter().map(|path| path.path))
            .collect()
    };
    Some(Loop { paths, wipe_paths })
}

#[expect(clippy::too_many_arguments)]
fn add_slope(
    mut path: ExtrusionPath,
    begin: f64,
    mut end: f64,
    max_segment: f64,
    seam_gap: f64,
    speed: f64,
    flow_ratio: f64,
    starts: &mut Vec<Path>,
    ends: &mut Vec<Path>,
) {
    path.polyline.points = detail(&path.polyline.points, max_segment);
    path.polyline.fitting.clear();
    starts.push(Path {
        path: path.clone(),
        slope: Some(Slope {
            z_begin: begin,
            z_end: end,
            e_begin: begin,
            e_end: end,
            speed,
            flow_ratio,
        }),
    });

    let length = path_length(&path.polyline.points);
    if (end - 1.0).abs() <= 1e-4 && seam_gap > 0.0 {
        if length > seam_gap {
            clip_end_points(&mut path.polyline.points, seam_gap);
            end = begin + (end - begin) * ((length - seam_gap) / length);
        } else {
            path.polyline.points.clear();
        }
    }
    if path.polyline.points.len() >= 2 {
        ends.push(Path {
            path,
            slope: Some(Slope {
                z_begin: 1.0,
                z_end: 1.0,
                e_begin: 1.0 - begin,
                e_end: 1.0 - end,
                speed,
                flow_ratio,
            }),
        });
    }
}

fn scarf_speed(role: ExtrusionRole, loop_length: f64, options: &MotionOptions) -> f64 {
    let base = if role == ExtrusionRole::ExternalPerimeter {
        options.outer_wall_speed
    } else {
        options.inner_wall_speed
    };
    if loop_length <= options.small_perimeter_threshold * 2.0 * std::f64::consts::PI {
        return options.small_perimeter_speed;
    }
    let configured = match options
        .scarf
        .speed
        .unwrap_or(FloatOrPercent::Percent(crate::Percent(100.0)))
    {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => base * value.0 / 100.0,
    };
    base.min(configured)
}

fn split_path(path: &ExtrusionPath, distance: f64) -> (ExtrusionPath, ExtrusionPath) {
    let points = &path.polyline.points;
    let mut traversed = 0.0;
    for (index, segment) in points.windows(2).enumerate() {
        let length = point_distance(segment[0], segment[1]);
        if traversed + length >= distance {
            let ratio = (distance - traversed) / length;
            let point = lerp(segment[0], segment[1], ratio);
            let mut before = points[..=index].to_vec();
            if before.last() != Some(&point) {
                before.push(point);
            }
            let mut after = vec![point];
            if point == segment[1] {
                after.clear();
            }
            after.extend_from_slice(&points[index + 1..]);
            return (
                copy_with_points(path, before),
                copy_with_points(path, after),
            );
        }
        traversed += length;
    }
    (
        path.clone(),
        copy_with_points(path, vec![*points.last().unwrap()]),
    )
}

fn copy_with_points(path: &ExtrusionPath, points: Vec<Point3>) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points,
            fitting: Vec::new(),
        },
        role: path.role,
        can_reverse: path.can_reverse,
        mm3_per_mm: path.mm3_per_mm,
        width: path.width,
        height: path.height,
    }
}

fn detail(points: &[Point3], maximum: f64) -> Vec<Point3> {
    let mut output = vec![points[0]];
    for segment in points.windows(2) {
        detail_segment(segment[0], segment[1], maximum, &mut output);
    }
    output
}

fn detail_segment(a: Point3, b: Point3, maximum: f64, output: &mut Vec<Point3>) {
    if point_distance(a, b) <= maximum {
        output.push(b);
    } else {
        let midpoint = Point3 {
            x: (a.x + b.x) / 2,
            y: (a.y + b.y) / 2,
            z: (a.z + b.z) / 2,
        };
        detail_segment(a, midpoint, maximum, output);
        detail_segment(midpoint, b, maximum, output);
    }
}

fn clip_front_paths(paths: &mut Vec<Path>, mut distance: f64) {
    while distance > 0.0 && !paths.is_empty() {
        let length = path_length(&paths[0].path.polyline.points);
        if length <= distance {
            paths.remove(0);
            distance -= length;
        } else {
            clip_front_points(&mut paths[0].path.polyline.points, distance);
            break;
        }
    }
}

fn clip_end_paths(paths: &mut Vec<Path>, mut distance: f64) {
    while distance > 0.0 && !paths.is_empty() {
        let index = paths.len() - 1;
        let length = path_length(&paths[index].path.polyline.points);
        if length <= distance {
            paths.pop();
            distance -= length;
        } else {
            clip_end_points(&mut paths[index].path.polyline.points, distance);
            break;
        }
    }
}

fn clip_front_points(points: &mut Vec<Point3>, distance: f64) {
    points.reverse();
    clip_end_points(points, distance);
    points.reverse();
}

fn clip_end_points(points: &mut Vec<Point3>, mut distance: f64) {
    while points.len() > 1 {
        let last = points[points.len() - 1];
        let previous = points[points.len() - 2];
        let length = point_distance(previous, last);
        if length > distance {
            let ratio = distance / length;
            *points.last_mut().unwrap() = lerp(last, previous, ratio);
            return;
        }
        points.pop();
        distance -= length;
        if distance <= 0.0 {
            return;
        }
    }
    points.clear();
}

fn path_length(points: &[Point3]) -> f64 {
    points
        .windows(2)
        .map(|segment| point_distance(segment[0], segment[1]))
        .sum()
}

fn point_distance(a: Point3, b: Point3) -> f64 {
    ((b.x - a.x) as f64).hypot((b.y - a.y) as f64)
}

fn lerp(a: Point3, b: Point3, ratio: f64) -> Point3 {
    Point3 {
        x: ((1.0 - ratio) * a.x as f64 + ratio * b.x as f64) as i64,
        y: ((1.0 - ratio) * a.y as f64 + ratio * b.y as f64) as i64,
        z: ((1.0 - ratio) * a.z as f64 + ratio * b.z as f64) as i64,
    }
}
