use crate::geometry::Point;

use super::segments::{
    IntersectionKind, LinkQuality, LinkType, SegmentIntersection, SegmentedLine,
};

pub(crate) fn insert_phony_outer_pairs(lines: &mut [SegmentedLine]) {
    for line_index in 1..lines.len() {
        let insert_after = pinch_positions(&lines[line_index]);
        if insert_after.is_empty() {
            continue;
        }
        let (intersections, map) = insert_pairs(&lines[line_index], &insert_after);
        lines[line_index].intersections = intersections;
        remap_current(&mut lines[line_index], &map);
        remap_previous(&mut lines[line_index - 1], &map);
        if line_index + 1 < lines.len() {
            remap_next(&mut lines[line_index + 1], &map);
        }
    }
}

fn pinch_positions(line: &SegmentedLine) -> Vec<usize> {
    line.intersections
        .windows(2)
        .enumerate()
        .filter_map(|(index, pair)| {
            (pair[0].kind == IntersectionKind::InnerHigh
                && pair[1].kind == IntersectionKind::InnerLow
                && !connected_pair(pair[0].next, pair[1].previous, index, index + 1))
            .then_some(index)
        })
        .collect()
}

fn connected_pair(
    high: Option<(usize, LinkType, LinkQuality)>,
    low: Option<(usize, LinkType, LinkQuality)>,
    high_index: usize,
    low_index: usize,
) -> bool {
    high.is_some_and(|link| {
        link.0 == low_index
            && matches!(link.1, LinkType::Up | LinkType::Down)
            && link.2 == LinkQuality::Valid
    }) || low.is_some_and(|link| {
        link.0 == high_index
            && matches!(link.1, LinkType::Up | LinkType::Down)
            && link.2 == LinkQuality::Valid
    })
}

fn insert_pairs(
    line: &SegmentedLine,
    insert_after: &[usize],
) -> (Vec<SegmentIntersection>, Vec<usize>) {
    let mut output = Vec::with_capacity(line.intersections.len() + 2 * insert_after.len());
    let mut map = Vec::with_capacity(line.intersections.len());
    let mut insertion = 0;
    for (index, intersection) in line.intersections.iter().copied().enumerate() {
        map.push(output.len());
        output.push(intersection);
        if insert_after.get(insertion) == Some(&index) {
            let next = line.intersections[index + 1];
            let y = (intersection.point.y() + next.point.y()) / 2;
            output.push(phony(line.x, y, IntersectionKind::OuterHigh));
            output.push(phony(line.x, y, IntersectionKind::OuterLow));
            insertion += 1;
        }
    }
    (output, map)
}

fn phony(x: i64, y: i64, kind: IntersectionKind) -> SegmentIntersection {
    SegmentIntersection {
        point: Point::new(x, y),
        contour_index: usize::MAX,
        segment_index: usize::MAX,
        kind,
        previous: None,
        next: None,
    }
}

fn remap_current(line: &mut SegmentedLine, map: &[usize]) {
    for intersection in &mut line.intersections {
        remap_vertical(&mut intersection.previous, map);
        remap_vertical(&mut intersection.next, map);
    }
}

fn remap_previous(line: &mut SegmentedLine, map: &[usize]) {
    for intersection in &mut line.intersections {
        remap_horizontal(&mut intersection.next, map);
    }
}

fn remap_next(line: &mut SegmentedLine, map: &[usize]) {
    for intersection in &mut line.intersections {
        remap_horizontal(&mut intersection.previous, map);
    }
}

fn remap_vertical(link: &mut Option<(usize, LinkType, LinkQuality)>, map: &[usize]) {
    if let Some((index, kind @ (LinkType::Up | LinkType::Down), quality)) = *link {
        *link = Some((map[index], kind, quality));
    }
}

fn remap_horizontal(link: &mut Option<(usize, LinkType, LinkQuality)>, map: &[usize]) {
    if let Some((index, LinkType::Horizontal, quality)) = *link {
        *link = Some((map[index], LinkType::Horizontal, quality));
    }
}
