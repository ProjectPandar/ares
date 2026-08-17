use crate::{
    SliceError,
    geometry::{CoordinateScale, Line, Point, ThickLine, ThickPolyline, medial_axis},
    project_slice::perimeters::types::Flow,
};

use super::{ExtrusionPath, ExtrusionRole, GapFillCollection, GapFillEntity, Point3, Polyline3};

pub(super) fn convert(
    polylines: &[ThickPolyline],
    flow: Flow,
    scale: CoordinateScale,
) -> Result<GapFillCollection, SliceError> {
    let tolerance = (0.05 / scale.factor()) as f32;
    let mut entities = Vec::new();
    for polyline in polylines {
        let paths = convert_polyline(polyline, flow, scale, f64::from(tolerance))?;
        if let (Some(first), Some(last)) = (paths.first(), paths.last()) {
            if first.polyline.points.first().map(|p| (p.x, p.y))
                == last.polyline.points.last().map(|p| (p.x, p.y))
            {
                entities.push(GapFillEntity::Loop(paths));
            } else {
                entities.extend(paths.into_iter().map(GapFillEntity::Path));
            }
        }
    }
    Ok(GapFillCollection { entities })
}

fn convert_polyline(
    polyline: &ThickPolyline,
    flow: Flow,
    scale: CoordinateScale,
    tolerance: f64,
) -> Result<Vec<ExtrusionPath>, SliceError> {
    let mut lines = polyline.thicklines();
    let epsilon = medial_axis::scaled_epsilon(scale);
    let mut paths = Vec::new();
    if lines.is_empty() {
        return Ok(paths);
    }
    let mut start_index = 0;
    let mut index = 0;
    let mut max_width = lines[0].a_width;
    let mut min_width = lines[0].a_width;

    while index < lines.len() {
        let line = lines[index];
        let line_len = Line::new(line.a, line.b).length();
        if line_len < epsilon {
            index += 1;
            continue;
        }

        let mut thickness_delta = (max_width - line.b_width)
            .abs()
            .max((min_width - line.b_width).abs());
        if thickness_delta > tolerance {
            if start_index != index {
                push_group(
                    &lines[start_index..index],
                    lines[index].a,
                    GroupContext {
                        flow,
                        scale,
                        epsilon,
                        final_group: false,
                    },
                    &mut paths,
                )?;
            }
            start_index = index;
            max_width = line.a_width;
            min_width = line.a_width;
            thickness_delta = (line.a_width - line.b_width).abs();
            if thickness_delta > tolerance {
                split_line(&mut lines, index, line, line_len, tolerance);
                continue;
            }
        } else {
            max_width = max_width.max(line.a_width.max(line.b_width));
            min_width = min_width.min(line.a_width.min(line.b_width));
        }
        index += 1;
    }

    if start_index < lines.len() {
        let endpoint = lines[lines.len() - 1].b;
        push_group(
            &lines[start_index..],
            endpoint,
            GroupContext {
                flow,
                scale,
                epsilon,
                final_group: true,
            },
            &mut paths,
        )?;
    }
    Ok(paths)
}

fn split_line(
    lines: &mut Vec<ThickLine>,
    index: usize,
    line: ThickLine,
    line_length: f64,
    tolerance: f64,
) {
    let segments = ((line.a_width - line.b_width).abs() / tolerance).ceil() as usize;
    let segment_length = line_length / segments as f64;
    let dx = (line.b.x() - line.a.x()) as f64;
    let dy = (line.b.y() - line.a.y()) as f64;
    let norm = (dx * dx + dy * dy).sqrt();
    let direction = (dx / norm, dy / norm);
    let mut points = Vec::with_capacity(segments + 1);
    let mut widths = Vec::with_capacity(segments * 2);
    points.push(line.a);
    widths.push(line.a_width);
    for segment in 1..segments {
        let distance = segment as f64 * segment_length;
        points.push(Point::new(
            (line.a.x() as f64 + direction.0 * distance) as i64,
            (line.a.y() as f64 + direction.1 * distance) as i64,
        ));
        let width = line.a_width + distance * (line.b_width - line.a_width) / line_length;
        widths.extend([width, width]);
    }
    points.push(line.b);
    widths.push(line.b_width);
    let replacements = (0..segments).map(|segment| {
        ThickLine::with_widths(
            points[segment],
            points[segment + 1],
            widths[2 * segment],
            widths[2 * segment + 1],
        )
    });
    lines.splice(index..=index, replacements);
}

#[derive(Clone, Copy)]
struct GroupContext {
    flow: Flow,
    scale: CoordinateScale,
    epsilon: f64,
    final_group: bool,
}

fn push_group(
    lines: &[ThickLine],
    endpoint: Point,
    context: GroupContext,
    output: &mut Vec<ExtrusionPath>,
) -> Result<(), SliceError> {
    let GroupContext {
        flow,
        scale,
        epsilon,
        final_group,
    } = context;
    let mut length = Line::new(lines[0].a, lines[0].b).length();
    let mut sum = length
        * if final_group {
            lines[0].a_width
        } else {
            0.5 * (lines[0].a_width + lines[0].b_width)
        };
    let mut points = Vec::with_capacity(lines.len() + 1);
    points.push(point3(lines[0].a));
    for line in &lines[1..] {
        let line_length = Line::new(line.a, line.b).length();
        length += line_length;
        sum += line_length
            * if final_group {
                line.a_width
            } else {
                0.5 * (line.a_width + line.b_width)
            };
        points.push(point3(line.a));
    }
    points.push(point3(endpoint));
    if length > epsilon {
        let scaled_width = sum / length;
        let width = scaled_width as f32 * scale.factor() as f32
            + flow.height * (1.0 - 0.25 * std::f64::consts::PI) as f32;
        let new_flow = flow.with_width(width)?;
        output.push(ExtrusionPath {
            polyline: Polyline3 {
                points,
                fitting: Vec::new(),
            },
            role: ExtrusionRole::GapFill,
            mm3_per_mm: new_flow.mm3_per_mm,
            width: new_flow.width,
            height: new_flow.height,
        });
    }
    Ok(())
}

const fn point3(point: Point) -> Point3 {
    Point3 {
        x: point.x(),
        y: point.y(),
        z: 0,
    }
}
