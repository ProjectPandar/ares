//! `GCode.cpp::extrude_loop` / `ExtrusionEntity.cpp::ExtrusionLoopSloped` scarf paths.

mod eligibility;
mod emit;
mod trim;

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
    (role, unsupported_distance): (ExtrusionLoopRole, Option<f32>),
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
    {
        return None;
    }
    if options.scarf.conditional
        && !eligibility::permits(
            original,
            unsupported_distance,
            options,
            geometry.scale.factor(),
        )
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
    let speed = scarf_speed(first.role, options);
    let max_segment = slope_length / options.scarf.steps as f64 / scale;
    let (original, slope_length) = trim::prepare(original, slope_length, scale);
    let seam_gap = options.seam_gap / scale;
    let mut starts = Vec::new();
    let mut flats = Vec::new();
    let mut ends = Vec::new();
    // `ExtrusionEntity.cpp:571-595`: keep remaining/path lengths in mm;
    // convert only the split distance to scaled coordinates.
    let mut remaining = slope_length;
    let mut ratio = start_ratio;
    let mut index = 0;

    while index < original.len() && remaining > 0.0 {
        let source = &original[index];
        let length = path_length(&source.polyline.points) * scale;
        let (slope_path, flat_path, end_ratio) = if length > remaining {
            let cut = (remaining / scale, (0.5 * max_segment).min(1.0 / scale));
            let (slope, flat) = split_path(source, cut);
            remaining = 0.0;
            (slope, (flat.polyline.points.len() > 1).then_some(flat), 1.0)
        } else {
            remaining -= length;
            let factor = remaining / slope_length;
            let end_ratio = (1.0 - factor) * 1.0 + factor * start_ratio;
            (source.clone(), None, end_ratio)
        };
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

    // `ExtrusionEntity.cpp:527-563`: `clip_front` is entered with
    // `distance = seam_gap * 2`; the erPerimeter `0.4 * slope_path_length`
    // clip_dist is only effective while that distance is positive — at
    // `seam_gap = 0` upstream emits the full slope head unclipped.
    if seam_gap * 2.0 > 0.0 {
        let front_clip = if flats
            .first()
            .is_some_and(|path| path.role == ExtrusionRole::Perimeter)
        {
            starts
                .iter()
                .map(|path| path_length(&path.path.polyline.points))
                .sum::<f64>()
                * 0.4
        } else {
            seam_gap * 2.0
        };
        clip_front_paths(&mut starts, front_clip);
    }
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

fn scarf_speed(role: ExtrusionRole, options: &MotionOptions) -> f64 {
    let base = if role == ExtrusionRole::ExternalPerimeter {
        options.outer_wall_speed
    } else {
        options.inner_wall_speed
    };
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

// `ExtrusionEntity.cpp:487-505`: split at the nearby vertex instead of leaving a short stub.
fn split_path(src: &ExtrusionPath, (distance, snap): (f64, f64)) -> (ExtrusionPath, ExtrusionPath) {
    let points = &src.polyline.points;
    let mut traversed = 0.0;
    for (index, segment) in points.windows(2).enumerate() {
        let length = point_distance(segment[0], segment[1]);
        let end_length = traversed + length;
        if end_length >= distance {
            let point = if end_length - distance < snap {
                segment[1]
            } else {
                lerp(segment[0], segment[1], (distance - traversed) / length)
            };
            let mut before = points[..=index].to_vec();
            if before.last() != Some(&point) {
                before.push(point);
            }
            let mut after = vec![point];
            if point == segment[1] {
                after.clear();
            }
            after.extend_from_slice(&points[index + 1..]);
            return (copy_with_points(src, before), copy_with_points(src, after));
        }
        traversed = end_length;
    }
    (
        src.clone(),
        copy_with_points(src, vec![*points.last().unwrap()]),
    )
}

fn copy_with_points(path: &ExtrusionPath, points: Vec<Point3>) -> ExtrusionPath {
    ExtrusionPath {
        z_contoured: path.z_contoured,
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

// `Polyline.cpp:683-702`: additive displacement before the coord_t cast.
fn clip_end_points(points: &mut Vec<Point3>, mut distance: f64) {
    while distance > 0.0 {
        let last = points.last().unwrap();
        let last = [last.x as f64, last.y as f64, last.z as f64];
        points.pop();
        let Some(previous) = points.last() else {
            return;
        };
        let v = [
            previous.x as f64 - last[0],
            previous.y as f64 - last[1],
            previous.z as f64 - last[2],
        ];
        let lsqr = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        if lsqr > distance * distance {
            let ratio = distance / lsqr.sqrt();
            points.push(Point3 {
                x: (last[0] + v[0] * ratio) as i64,
                y: (last[1] + v[1] * ratio) as i64,
                z: (last[2] + v[2] * ratio) as i64,
            });
            break;
        }
        distance -= lsqr.sqrt();
    }
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
