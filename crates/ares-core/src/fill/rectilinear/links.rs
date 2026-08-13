use super::segments::{
    IntersectionKind, LinkQuality, LinkType, SegmentIntersection, SegmentedLine,
};

pub(crate) fn connect_contours(
    lines: &mut [SegmentedLine],
    dont_connect: bool,
    maximum_length: f64,
) {
    let snapshot = lines.to_vec();
    for line_index in 0..lines.len() {
        for intersection_index in 0..lines[line_index].intersections.len() {
            let current = snapshot[line_index].intersections[intersection_index];
            let previous = candidate(
                &snapshot,
                line_index,
                intersection_index,
                current,
                Direction::Previous,
            );
            let next = candidate(
                &snapshot,
                line_index,
                intersection_index,
                current,
                Direction::Next,
            );
            lines[line_index].intersections[intersection_index].previous = previous.map(|link| {
                finish_quality(
                    &snapshot,
                    line_index,
                    current,
                    link,
                    Direction::Previous,
                    dont_connect,
                    maximum_length,
                )
            });
            lines[line_index].intersections[intersection_index].next = next.map(|link| {
                finish_quality(
                    &snapshot,
                    line_index,
                    current,
                    link,
                    Direction::Next,
                    dont_connect,
                    maximum_length,
                )
            });
        }
    }
    make_vertical_invalid_symmetric(lines);
}

#[derive(Clone, Copy)]
enum Direction {
    Previous,
    Next,
}

fn candidate(
    lines: &[SegmentedLine],
    line_index: usize,
    intersection_index: usize,
    current: SegmentIntersection,
    direction: Direction,
) -> Option<(usize, LinkType, LinkQuality)> {
    let adjacent_index = match direction {
        Direction::Previous => line_index.checked_sub(1),
        Direction::Next => (line_index + 1 < lines.len()).then_some(line_index + 1),
    };
    let mut best = adjacent_index.and_then(|adjacent| {
        lines[adjacent]
            .intersections
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.contour_index == current.contour_index && item.kind == current.kind
            })
            .min_by_key(|(_, item)| segment_distance(current, **item, direction))
            .map(|(index, item)| {
                (
                    index,
                    LinkType::Horizontal,
                    LinkQuality::Valid,
                    segment_distance(current, *item, direction),
                )
            })
    });
    for (index, item) in lines[line_index].intersections.iter().enumerate() {
        if index == intersection_index
            || item.contour_index != current.contour_index
            || is_low(item.kind) == is_low(current.kind)
        {
            continue;
        }
        let distance = segment_distance(current, *item, direction);
        if best.is_none_or(|best| distance < best.3) {
            best = Some((
                index,
                if index < intersection_index {
                    LinkType::Down
                } else {
                    LinkType::Up
                },
                LinkQuality::Valid,
                distance,
            ));
        }
    }
    best.map(|(index, kind, quality, _)| (index, kind, quality))
}

fn segment_distance(
    current: SegmentIntersection,
    other: SegmentIntersection,
    direction: Direction,
) -> usize {
    match direction {
        Direction::Previous => current.segment_index.abs_diff(other.segment_index),
        Direction::Next => other.segment_index.abs_diff(current.segment_index),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "source link quality depends on line, point, direction, and both connection gates"
)]
fn finish_quality(
    lines: &[SegmentedLine],
    line_index: usize,
    current: SegmentIntersection,
    (index, kind, quality): (usize, LinkType, LinkQuality),
    direction: Direction,
    dont_connect: bool,
    maximum_length: f64,
) -> (usize, LinkType, LinkQuality) {
    let quality = if dont_connect {
        LinkQuality::TooLong
    } else if maximum_length > 0.0 {
        let target_line = match (kind, direction) {
            (LinkType::Horizontal, Direction::Previous) => line_index - 1,
            (LinkType::Horizontal, Direction::Next) => line_index + 1,
            (LinkType::Up | LinkType::Down, _) => line_index,
        };
        let target = lines[target_line].intersections[index].point;
        let dx = (target.x() - current.point.x()) as f64;
        let dy = (target.y() - current.point.y()) as f64;
        if (dx * dx + dy * dy).sqrt() > maximum_length {
            LinkQuality::TooLong
        } else {
            quality
        }
    } else {
        quality
    };
    (index, kind, quality)
}

#[expect(
    clippy::excessive_nesting,
    reason = "source symmetry pass walks lines, intersections, then both directional links"
)]
fn make_vertical_invalid_symmetric(lines: &mut [SegmentedLine]) {
    for line in lines {
        let snapshot = line.intersections.clone();
        for item in &snapshot {
            for link in [item.previous, item.next].into_iter().flatten() {
                if !matches!(link.1, LinkType::Up | LinkType::Down)
                    || link.2 != LinkQuality::Invalid
                {
                    continue;
                }
                let target = &mut line.intersections[link.0];
                if target
                    .previous
                    .is_some_and(|candidate| candidate.0 == link.0)
                {
                    target.previous = target
                        .previous
                        .map(|(index, kind, _)| (index, kind, LinkQuality::Invalid));
                }
                if target.next.is_some_and(|candidate| candidate.0 == link.0) {
                    target.next = target
                        .next
                        .map(|(index, kind, _)| (index, kind, LinkQuality::Invalid));
                }
            }
        }
    }
}

const fn is_low(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::OuterLow | IntersectionKind::InnerLow
    )
}
