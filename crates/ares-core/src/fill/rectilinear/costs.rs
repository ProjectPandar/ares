use crate::geometry::{CoordinateScale, Point};

use super::perimeter::measure_horizontal_arc;
use super::regions::{MonotonicRegion, vertical_run_bottom, vertical_run_top};
use super::segments::{
    IntersectionKind, LinkQuality, LinkType, RectilinearSlice, SegmentIntersection,
};

pub(crate) fn compute_region_costs(
    regions: &mut [MonotonicRegion],
    slice: &RectilinearSlice,
    scale: CoordinateScale,
) {
    for region in regions {
        let first = region_path_cost(region, false, slice, scale);
        let second = region_path_cost(region, true, slice, scale);
        region.lengths = if first > second {
            [first - second, 0.0]
        } else {
            [0.0, second - first]
        };
    }
}

#[expect(
    clippy::excessive_nesting,
    reason = "source traversal walks regions, vertical runs, and linked inner contour segments"
)]
fn region_path_cost(
    region: &MonotonicRegion,
    flipped: bool,
    slice: &RectilinearSlice,
    scale: CoordinateScale,
) -> f32 {
    let mut intersection = if flipped {
        region.left.high
    } else {
        region.left.low
    };
    let mut line_index = region.left.line;
    let mut total = 0.0_f32;
    let mut split_gap = false;
    let mut last = Point::new(0, 0);

    loop {
        let line = &slice.lines[line_index];
        let mut index = intersection;
        let going_up = is_low(line.intersections[index].kind);
        if split_gap {
            let outer = line.intersections[if going_up { index - 1 } else { index + 1 }].point;
            total += point_distance_f32(last, outer);
        }

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
                let Some((target, LinkQuality::Valid)) =
                    vertical_link(line.intersections[index], LinkType::Up)
                else {
                    break;
                };
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
                let Some((target, LinkQuality::Valid)) =
                    vertical_link(line.intersections[index], LinkType::Down)
                else {
                    break;
                };
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
        let current_right = right_horizontal(line.intersections[index]);
        if current_right == Some((intersection, LinkQuality::Valid)) {
            total +=
                0.5_f32 * measure_horizontal_arc(slice, line_index, index, intersection) as f32;
            split_gap = false;
        } else {
            let outer = line.intersections[if going_up { index + 1 } else { index - 1 }].point;
            last = outer;
            split_gap = true;
        }
        line_index += 1;
    }

    (f64::from(total) * scale.factor()) as f32
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

fn point_distance_f32(first: Point, second: Point) -> f32 {
    let x = (first.x() - second.x()) as f32;
    let y = (first.y() - second.y()) as f32;
    x.hypot(y)
}

const fn is_low(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::OuterLow | IntersectionKind::InnerLow
    )
}
