use crate::geometry::{CoordinateScale, Point, Polyline};

use super::chain::MonotonicRegionLink;
use super::perimeter::{emit_horizontal_arc, emit_vertical_arc};
use super::regions::{MonotonicRegion, vertical_run_bottom, vertical_run_top};
use super::segments::{
    IntersectionKind, LinkQuality, LinkType, RectilinearSlice, SegmentIntersection,
};

#[expect(
    clippy::excessive_nesting,
    reason = "source emitter nests path regions, vertical lines, inner runs, and contour links"
)]
pub(crate) fn emit_monotonic_polylines(
    path: &[MonotonicRegionLink],
    regions: &[MonotonicRegion],
    slice: &RectilinearSlice,
    scale: CoordinateScale,
) -> Vec<Polyline> {
    if path.is_empty() {
        return Vec::new();
    }
    let epsilon = 1.0e-4 / scale.factor();
    let mut output = Vec::new();
    let mut current: Option<Vec<Point>> = None;

    for (path_index, link) in path.iter().enumerate() {
        let region = &regions[link.region];
        let mut intersection = region.left_intersection(link.flipped);
        let mut line_index = region.left.line;

        if let Some(points) = &mut current
            && path_index > 0
        {
            let previous_link = path[path_index - 1];
            let previous_region = &regions[previous_link.region];
            let previous_line = previous_region.right.line;
            let previous_intersection = previous_region.right_intersection(previous_link.flipped);
            let previous = slice.lines[previous_line].intersections[previous_intersection];
            if previous_line + 1 == line_index
                && right_horizontal(previous) == Some((intersection, LinkQuality::Valid))
            {
                emit_horizontal_arc(
                    slice,
                    previous_line,
                    previous_intersection,
                    intersection,
                    true,
                    points,
                );
            } else {
                let outer = slice.lines[previous_line].intersections[if is_low(previous.kind) {
                    previous_intersection - 1
                } else {
                    previous_intersection + 1
                }]
                .point;
                *points.last_mut().expect("active polyline has an endpoint") = outer;
                finish_polyline(&mut current, &mut output, epsilon);
            }
        }

        loop {
            let line = &slice.lines[line_index];
            let mut index = intersection;
            let going_up = is_low(line.intersections[index].kind);
            if let Some(points) = &mut current {
                points.push(line.intersections[index].point);
            } else {
                current = Some(vec![
                    line.intersections[if going_up { index - 1 } else { index + 1 }].point,
                ]);
            }
            let points = current.as_mut().expect("polyline was initialized");
            let mut right = right_horizontal(line.intersections[index]).map(|link| link.0);

            if going_up {
                loop {
                    loop {
                        index += 1;
                        if let Some((target, _)) = right_horizontal(line.intersections[index]) {
                            right = Some(right.map_or(target, |current| current.max(target)));
                        }
                        if line.intersections[index].kind == IntersectionKind::InnerHigh
                            && line.intersections[index + 1].kind == IntersectionKind::OuterHigh
                        {
                            break;
                        }
                    }
                    points.push(line.intersections[index].point);
                    let item = line.intersections[index];
                    let Some((target, LinkQuality::Valid)) = vertical_link(item, LinkType::Up)
                    else {
                        break;
                    };
                    emit_vertical_arc(
                        slice,
                        line_index,
                        index,
                        target,
                        item.previous.is_some_and(|link| link.1 == LinkType::Up),
                        points,
                    );
                    index = target;
                }
            } else {
                loop {
                    loop {
                        index -= 1;
                        if let Some((target, _)) = right_horizontal(line.intersections[index]) {
                            right = Some(target);
                        }
                        if line.intersections[index].kind == IntersectionKind::InnerLow
                            && line.intersections[index - 1].kind == IntersectionKind::OuterLow
                        {
                            break;
                        }
                    }
                    points.push(line.intersections[index].point);
                    let item = line.intersections[index];
                    let Some((target, LinkQuality::Valid)) = vertical_link(item, LinkType::Down)
                    else {
                        break;
                    };
                    emit_vertical_arc(
                        slice,
                        line_index,
                        index,
                        target,
                        item.next.is_some_and(|link| link.1 == LinkType::Down),
                        points,
                    );
                    index = target;
                }
            }

            if line_index == region.right.line {
                break;
            }
            let target = right.expect("region overlap has a right horizontal target");
            let next_line = &slice.lines[line_index + 1];
            intersection = if going_up {
                vertical_run_top(next_line, target)
            } else {
                vertical_run_bottom(next_line, target)
            };
            if right_horizontal(line.intersections[index])
                == Some((intersection, LinkQuality::Valid))
            {
                emit_horizontal_arc(slice, line_index, index, intersection, true, points);
            } else {
                let outer = line.intersections[if going_up { index + 1 } else { index - 1 }].point;
                *points.last_mut().expect("active polyline has an endpoint") = outer;
                finish_polyline(&mut current, &mut output, epsilon);
            }
            line_index += 1;
        }
    }

    if let Some(points) = &mut current {
        let last = *path.last().expect("path is nonempty");
        let region = &regions[last.region];
        let line = &slice.lines[region.right.line];
        let intersection = region.right_intersection(last.flipped);
        let item = line.intersections[intersection];
        let outer = line.intersections[if is_low(item.kind) {
            intersection - 1
        } else {
            intersection + 1
        }]
        .point;
        *points.last_mut().expect("active polyline has an endpoint") = outer;
        finish_polyline(&mut current, &mut output, epsilon);
    }
    output.into_iter().map(Polyline::new).collect()
}

fn finish_polyline(current: &mut Option<Vec<Point>>, output: &mut Vec<Vec<Point>>, epsilon: f64) {
    let mut points = current.take().expect("active polyline exists");
    points.dedup();
    if points.len() <= 1 || points.len() == 2 && points_close(points[0], points[1], epsilon) {
        return;
    }
    if let Some(previous) = output.last_mut()
        && points_close(
            *points.first().expect("valid polyline has a first point"),
            *previous.last().expect("valid polyline has a last point"),
            epsilon,
        )
    {
        let first = points[0];
        let last = previous
            .last_mut()
            .expect("valid polyline has a last point");
        *last = Point::new((last.x() + first.x()) / 2, (last.y() + first.y()) / 2);
        previous.extend(points.into_iter().skip(1));
    } else {
        output.push(points);
    }
}

fn points_close(first: Point, second: Point, epsilon: f64) -> bool {
    ((first.x() - second.x()).abs() as f64) < epsilon
        && ((first.y() - second.y()).abs() as f64) < epsilon
}

fn right_horizontal(item: SegmentIntersection) -> Option<(usize, LinkQuality)> {
    item.next.and_then(|(target, kind, quality)| {
        (kind == LinkType::Horizontal).then_some((target, quality))
    })
}

fn vertical_link(item: SegmentIntersection, direction: LinkType) -> Option<(usize, LinkQuality)> {
    [item.previous, item.next]
        .into_iter()
        .flatten()
        .find_map(|(target, kind, quality)| (kind == direction).then_some((target, quality)))
}

const fn is_low(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::OuterLow | IntersectionKind::InnerLow
    )
}
