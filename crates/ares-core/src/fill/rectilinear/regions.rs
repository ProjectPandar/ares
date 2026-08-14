use super::segments::{IntersectionKind, LinkQuality, LinkType, SegmentedLine};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionBoundary {
    pub(crate) line: usize,
    pub(crate) low: usize,
    pub(crate) high: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MonotonicRegion {
    pub(crate) left: RegionBoundary,
    pub(crate) right: RegionBoundary,
    pub(crate) flips: bool,
    pub(crate) left_neighbors: Vec<usize>,
    pub(crate) right_neighbors: Vec<usize>,
    pub(crate) lengths: [f32; 2],
}

impl MonotonicRegion {
    pub(crate) const fn left_intersection(&self, flipped: bool) -> usize {
        if flipped {
            self.left.high
        } else {
            self.left.low
        }
    }

    pub(crate) const fn right_intersection(&self, flipped: bool) -> usize {
        if flipped == self.flips {
            self.right.low
        } else {
            self.right.high
        }
    }
}

#[expect(
    clippy::excessive_nesting,
    reason = "source region generation scans seed lines, inner runs, then exclusive right overlaps"
)]
pub(crate) fn generate_monotonic_regions(lines: &[SegmentedLine]) -> Vec<MonotonicRegion> {
    let mut consumed = lines
        .iter()
        .map(|line| vec![false; line.intersections.len()])
        .collect::<Vec<_>>();
    let mut regions = Vec::new();
    for seed_line in 0..lines.len() {
        let mut seed = 1;
        while seed + 1 < lines[seed_line].intersections.len() {
            while seed < lines[seed_line].intersections.len()
                && lines[seed_line].intersections[seed].kind != IntersectionKind::InnerLow
            {
                seed += 1;
            }
            if seed == lines[seed_line].intersections.len() {
                break;
            }
            let high = vertical_run_top(&lines[seed_line], seed);
            if !consumed[seed_line][seed] {
                let mut left = (seed, high);
                let mut right = left;
                let mut right_line = seed_line;
                consumed[seed_line][seed] = true;
                let mut count = 1;
                while right_line + 1 < lines.len() {
                    let Some(candidate) =
                        overlap_right(&lines[right_line], &lines[right_line + 1], left)
                    else {
                        break;
                    };
                    if vertical_run_top(&lines[right_line + 1], candidate.0) != candidate.1 {
                        break;
                    }
                    let Some(back) =
                        overlap_left(&lines[right_line + 1], &lines[right_line], candidate)
                    else {
                        break;
                    };
                    if back != left {
                        break;
                    }
                    right_line += 1;
                    right = candidate;
                    consumed[right_line][right.0] = true;
                    count += 1;
                    left = right;
                }
                regions.push(MonotonicRegion {
                    left: RegionBoundary {
                        line: seed_line,
                        low: seed,
                        high,
                    },
                    right: RegionBoundary {
                        line: right_line,
                        low: right.0,
                        high: right.1,
                    },
                    flips: count % 2 == 1,
                    left_neighbors: Vec::new(),
                    right_neighbors: Vec::new(),
                    lengths: [0.0; 2],
                });
            }
            seed = high + 1;
        }
    }
    regions
}

fn overlap_right(
    current: &SegmentedLine,
    next: &SegmentedLine,
    (low, high): (usize, usize),
) -> Option<(usize, usize)> {
    overlap(current, next, low, high, |item| item.next)
}

fn overlap_left(
    current: &SegmentedLine,
    previous: &SegmentedLine,
    (low, high): (usize, usize),
) -> Option<(usize, usize)> {
    overlap(current, previous, low, high, |item| item.previous)
}

fn overlap(
    current: &SegmentedLine,
    other: &SegmentedLine,
    low: usize,
    high: usize,
    link: impl Fn(&super::segments::SegmentIntersection) -> Option<(usize, LinkType, LinkQuality)>,
) -> Option<(usize, usize)> {
    let linked = (low..=high).find_map(|index| {
        link(&current.intersections[index])
            .and_then(|(target, kind, _)| (kind == LinkType::Horizontal).then_some(target))
    })?;
    let bottom = vertical_run_bottom(other, linked);
    let top = vertical_run_top(other, linked);
    (bottom < top).then_some((bottom, top))
}

pub(super) fn vertical_run_bottom(line: &SegmentedLine, mut index: usize) -> usize {
    loop {
        while index > 0 && line.intersections[index].kind != IntersectionKind::InnerLow {
            index -= 1;
        }
        if index > 0 && line.intersections[index - 1].kind == IntersectionKind::InnerHigh {
            index -= 1;
            continue;
        }
        let Some((target, kind, quality)) = line.intersections[index].previous else {
            break;
        };
        if kind != LinkType::Down || quality != LinkQuality::Valid {
            break;
        }
        index = target;
    }
    index
}

pub(super) fn vertical_run_top(line: &SegmentedLine, mut index: usize) -> usize {
    loop {
        while index + 1 < line.intersections.len()
            && line.intersections[index].kind != IntersectionKind::InnerHigh
        {
            index += 1;
        }
        if index + 1 < line.intersections.len()
            && line.intersections[index + 1].kind == IntersectionKind::InnerLow
        {
            index += 1;
            continue;
        }
        let Some((target, kind, quality)) = line.intersections[index].next else {
            break;
        };
        if kind != LinkType::Up || quality != LinkQuality::Valid {
            break;
        }
        index = target;
    }
    index
}
