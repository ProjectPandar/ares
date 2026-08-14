use super::perimeter::{directed_segment_distance, measure_horizontal_arc, measure_vertical_arc};
use super::segments::{
    IntersectionKind, LinkQuality, LinkType, RectilinearSlice, SegmentIntersection,
};

pub(crate) fn connect_contours(
    slice: &mut RectilinearSlice,
    dont_connect: bool,
    maximum_length: f64,
) {
    let snapshot = slice.clone();
    for line_index in 0..snapshot.lines.len() {
        for intersection_index in 0..snapshot.lines[line_index].intersections.len() {
            let (previous, next) = select_links(&snapshot, line_index, intersection_index);
            slice.lines[line_index].intersections[intersection_index].previous =
                Some(finish_quality(
                    &snapshot,
                    line_index,
                    intersection_index,
                    previous,
                    Direction::Previous,
                    dont_connect,
                    maximum_length,
                ));
            slice.lines[line_index].intersections[intersection_index].next = Some(finish_quality(
                &snapshot,
                line_index,
                intersection_index,
                next,
                Direction::Next,
                dont_connect,
                maximum_length,
            ));
        }
    }
    make_vertical_invalid_symmetric(&mut slice.lines);
}

#[derive(Clone, Copy)]
enum Direction {
    Previous,
    Next,
}

#[derive(Clone, Copy)]
struct Candidate {
    index: usize,
    kind: LinkType,
    distance: usize,
    same_line: bool,
    quality: LinkQuality,
}

fn select_links(
    slice: &RectilinearSlice,
    line_index: usize,
    intersection_index: usize,
) -> (Candidate, Candidate) {
    let line = &slice.lines[line_index];
    let current = line.intersections[intersection_index];
    let point_count = slice.contours[current.contour_index].polygon.points().len();
    let forward = is_low(current.kind);
    let mut previous = adjacent_candidate(
        slice,
        line_index.checked_sub(1),
        current,
        point_count,
        forward,
        Direction::Previous,
    );
    let mut next = adjacent_candidate(
        slice,
        (line_index + 1 < slice.lines.len()).then_some(line_index + 1),
        current,
        point_count,
        forward,
        Direction::Next,
    );

    for (index, item) in line.intersections.iter().copied().enumerate() {
        if index == intersection_index
            || item.contour_index != current.contour_index
            || item.kind == current.kind
        {
            continue;
        }
        let previous_distance = directed_segment_distance(
            point_count,
            item.segment_index,
            current.segment_index,
            forward,
        );
        if previous.is_none_or(|candidate| previous_distance < candidate.distance) {
            previous = Some(vertical_candidate(
                index,
                intersection_index,
                previous_distance,
            ));
        }
        let next_distance = directed_segment_distance(
            point_count,
            current.segment_index,
            item.segment_index,
            forward,
        );
        if next.is_none_or(|candidate| next_distance < candidate.distance) {
            next = Some(vertical_candidate(index, intersection_index, next_distance));
        }
    }

    let mut previous = previous.expect("closed contours have a previous intersection");
    let mut next = next.expect("closed contours have a next intersection");
    invalidate_skipped_inner(
        line.intersections.as_slice(),
        intersection_index,
        &mut previous,
    );
    invalidate_skipped_inner(line.intersections.as_slice(), intersection_index, &mut next);
    if previous.same_line
        && next.same_line
        && (previous.index > intersection_index) == (next.index > intersection_index)
    {
        previous.quality = LinkQuality::Invalid;
        next.quality = LinkQuality::Invalid;
    }
    (previous, next)
}

#[expect(
    clippy::too_many_arguments,
    reason = "source adjacent search carries contour direction and current intersection state"
)]
fn adjacent_candidate(
    slice: &RectilinearSlice,
    adjacent_line: Option<usize>,
    current: SegmentIntersection,
    point_count: usize,
    forward: bool,
    direction: Direction,
) -> Option<Candidate> {
    adjacent_line.and_then(|line_index| {
        slice.lines[line_index]
            .intersections
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.contour_index == current.contour_index && item.kind == current.kind
            })
            .map(|(index, item)| Candidate {
                index,
                kind: LinkType::Horizontal,
                distance: match direction {
                    Direction::Previous => directed_segment_distance(
                        point_count,
                        item.segment_index,
                        current.segment_index,
                        forward,
                    ),
                    Direction::Next => directed_segment_distance(
                        point_count,
                        current.segment_index,
                        item.segment_index,
                        forward,
                    ),
                },
                same_line: false,
                quality: LinkQuality::Valid,
            })
            .min_by_key(|candidate| candidate.distance)
    })
}

const fn vertical_candidate(index: usize, current: usize, distance: usize) -> Candidate {
    Candidate {
        index,
        kind: if index < current {
            LinkType::Down
        } else {
            LinkType::Up
        },
        distance,
        same_line: true,
        quality: LinkQuality::Valid,
    }
}

fn invalidate_skipped_inner(
    intersections: &[SegmentIntersection],
    current: usize,
    candidate: &mut Candidate,
) {
    if !candidate.same_line {
        return;
    }
    let (low, high) = if current < candidate.index {
        (current, candidate.index)
    } else {
        (candidate.index, current)
    };
    if intersections[low + 1..high]
        .iter()
        .any(|item| is_inner(item.kind))
    {
        candidate.quality = LinkQuality::Invalid;
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "source quality gate consumes indexed link direction and both connection options"
)]
fn finish_quality(
    slice: &RectilinearSlice,
    line: usize,
    intersection: usize,
    candidate: Candidate,
    direction: Direction,
    dont_connect: bool,
    maximum_length: f64,
) -> (usize, LinkType, LinkQuality) {
    let mut quality = candidate.quality;
    if quality == LinkQuality::Valid && dont_connect {
        quality = LinkQuality::TooLong;
    } else if quality == LinkQuality::Valid && maximum_length > 0.0 {
        let forward = is_low(slice.lines[line].intersections[intersection].kind);
        let length = match (candidate.same_line, direction) {
            (true, Direction::Previous) => {
                measure_vertical_arc(slice, line, candidate.index, intersection, forward)
            }
            (true, Direction::Next) => {
                measure_vertical_arc(slice, line, intersection, candidate.index, forward)
            }
            (false, Direction::Previous) => {
                measure_horizontal_arc(slice, line - 1, candidate.index, intersection)
            }
            (false, Direction::Next) => {
                measure_horizontal_arc(slice, line, intersection, candidate.index)
            }
        };
        if length > maximum_length {
            quality = LinkQuality::TooLong;
        }
    }
    (candidate.index, candidate.kind, quality)
}

fn make_vertical_invalid_symmetric(lines: &mut [super::segments::SegmentedLine]) {
    for line in lines {
        let snapshot = line.intersections.clone();
        for item in snapshot {
            if let Some((target, kind, LinkQuality::Invalid)) = item.previous
                && matches!(kind, LinkType::Up | LinkType::Down)
            {
                line.intersections[target].previous = line.intersections[target]
                    .previous
                    .map(|(index, kind, _)| (index, kind, LinkQuality::Invalid));
            }
            if let Some((target, kind, LinkQuality::Invalid)) = item.next
                && matches!(kind, LinkType::Up | LinkType::Down)
            {
                line.intersections[target].next = line.intersections[target]
                    .next
                    .map(|(index, kind, _)| (index, kind, LinkQuality::Invalid));
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

const fn is_inner(kind: IntersectionKind) -> bool {
    matches!(
        kind,
        IntersectionKind::InnerLow | IntersectionKind::InnerHigh
    )
}
