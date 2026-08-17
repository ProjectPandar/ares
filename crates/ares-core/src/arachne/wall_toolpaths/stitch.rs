use std::collections::HashMap;

use crate::{arachne::extrusion_line::ExtrusionLine, geometry::Point};

#[derive(Clone, Copy)]
struct Endpoint {
    line_index: usize,
    at_start: bool,
}

struct CandidateContext<'a> {
    chain: &'a ExtrusionLine,
    chain_length: i64,
    reverse_direction: bool,
    should_close: bool,
}

struct StitchState<'a> {
    lines: &'a [ExtrusionLine],
    processed: Vec<bool>,
    grid: HashMap<(i64, i64), Vec<Endpoint>>,
    max_distance: i64,
    snap_distance: i64,
}

impl StitchState<'_> {
    fn closest(
        &self,
        chain: &ExtrusionLine,
        chain_length: i64,
        reverse_direction: bool,
        should_close: bool,
    ) -> Option<(Endpoint, bool)> {
        let from = chain.junctions.last().unwrap().point;
        let min_x = (from.x() - self.max_distance) / self.max_distance;
        let max_x = (from.x() + self.max_distance) / self.max_distance;
        let min_y = (from.y() - self.max_distance) / self.max_distance;
        let max_y = (from.y() + self.max_distance) / self.max_distance;
        let endpoints = (min_y..=max_y).flat_map(|cell_y| {
            (min_x..=max_x).flat_map(move |cell_x| {
                self.grid
                    .get(&(cell_x, cell_y))
                    .into_iter()
                    .flatten()
                    .copied()
            })
        });
        let context = CandidateContext {
            chain,
            chain_length,
            reverse_direction,
            should_close,
        };
        let mut closest = None;
        let mut closest_distance = i64::MAX;
        for endpoint in endpoints {
            let Some((candidate_distance, closes)) = self.evaluate_candidate(endpoint, &context)
            else {
                continue;
            };
            if candidate_distance < closest_distance {
                closest_distance = candidate_distance;
                closest = Some((endpoint, closes));
            }
            if candidate_distance < self.snap_distance {
                break;
            }
        }
        closest
    }

    fn evaluate_candidate(
        &self,
        endpoint: Endpoint,
        context: &CandidateContext<'_>,
    ) -> Option<(i64, bool)> {
        let point = endpoint.point(self.lines);
        let from = context.chain.junctions.last().unwrap().point;
        let mut candidate_distance = distance(point, from);
        if candidate_distance > self.max_distance {
            return None;
        }
        let closes = squared_distance(point, context.chain.junctions.first().unwrap().point)
            < i128::from(self.snap_distance) * i128::from(self.snap_distance);
        if closes {
            if context.chain_length + candidate_distance < 3 * self.max_distance
                || context.chain.junctions.len() <= 2
            {
                return None;
            }
            candidate_distance += if context.should_close {
                -self.snap_distance
            } else {
                self.snap_distance
            };
        } else if self.processed[endpoint.line_index] {
            return None;
        }
        let candidate = &self.lines[endpoint.line_index];
        let would_reverse = (!endpoint.at_start) ^ context.reverse_direction;
        if (!candidate.is_odd && would_reverse) || candidate.is_odd != context.chain.is_odd {
            return None;
        }
        Some((candidate_distance, closes))
    }

    fn extend_direction(
        &mut self,
        chain: &mut ExtrusionLine,
        reverse_direction: bool,
        should_close: &mut bool,
    ) -> bool {
        if reverse_direction {
            chain.reverse();
        }
        let mut chain_length = chain.length();
        loop {
            let Some((endpoint, closes)) =
                self.closest(chain, chain_length, reverse_direction, *should_close)
            else {
                return false;
            };
            if closes {
                return true;
            }
            let candidate = &self.lines[endpoint.line_index];
            let old_size = chain.junctions.len();
            append_candidate(chain, candidate, endpoint.at_start, self.snap_distance);
            chain_length += chain.junctions[old_size.saturating_sub(1)..]
                .windows(2)
                .map(|pair| distance(pair[0].point, pair[1].point))
                .sum::<i64>();
            *should_close &= !candidate.is_odd;
            assert!(!self.processed[endpoint.line_index]);
            self.processed[endpoint.line_index] = true;
        }
    }
}

impl Endpoint {
    fn point(self, lines: &[ExtrusionLine]) -> Point {
        let line = &lines[self.line_index];
        if self.at_start {
            line.junctions.first().unwrap().point
        } else {
            line.junctions.last().unwrap().point
        }
    }
}

pub(super) fn stitch_toolpaths(
    toolpaths: &mut [Vec<ExtrusionLine>],
    max_distance: i64,
    snap_distance: i64,
) {
    for lines in toolpaths {
        stitch_inset(lines, max_distance, snap_distance);
    }
}

fn stitch_inset(lines: &mut Vec<ExtrusionLine>, max_distance: i64, snap_distance: i64) {
    if lines.is_empty() {
        return;
    }
    let mut grid = HashMap::<(i64, i64), Vec<Endpoint>>::new();
    for line_index in 0..lines.len() {
        for at_start in [true, false] {
            let endpoint = Endpoint {
                line_index,
                at_start,
            };
            let point = endpoint.point(lines);
            grid.entry((point.x() / max_distance, point.y() / max_distance))
                .or_default()
                .push(endpoint);
        }
    }
    let mut state = StitchState {
        lines,
        processed: vec![false; lines.len()],
        grid,
        max_distance,
        snap_distance,
    };
    let mut stitched = Vec::new();
    let mut closed = Vec::new();
    for line_index in 0..state.lines.len() {
        if state.processed[line_index] {
            continue;
        }
        state.processed[line_index] = true;
        let original_is_odd = state.lines[line_index].is_odd;
        let mut should_close = original_is_odd;
        let mut chain = state.lines[line_index].clone();
        let mut closes = false;
        for reverse_direction in [false, true] {
            closes = state.extend_direction(&mut chain, reverse_direction, &mut should_close);
            if closes {
                restore_closed_chain_direction(&mut chain, reverse_direction);
                break;
            }
        }
        if closes {
            closed.push(chain);
        } else {
            if !original_is_odd {
                chain.reverse();
            }
            stitched.push(chain);
        }
    }
    drop(state);
    for mut polygon in closed {
        if polygon.junctions.is_empty() {
            continue;
        }
        let first = *polygon.junctions.first().unwrap();
        let last = *polygon.junctions.last().unwrap();
        if first.point != last.point && distance(first.point, last.point) < max_distance {
            polygon.push(first);
        }
        polygon.is_closed = true;
        stitched.push(polygon);
    }
    *lines = stitched;
}

fn restore_closed_chain_direction(chain: &mut ExtrusionLine, reverse_direction: bool) {
    if reverse_direction {
        chain.reverse();
    }
}

fn append_candidate(
    chain: &mut ExtrusionLine,
    candidate: &ExtrusionLine,
    at_start: bool,
    snap_distance: i64,
) {
    let candidate_endpoint = if at_start {
        candidate.junctions.first().unwrap().point
    } else {
        candidate.junctions.last().unwrap().point
    };
    let skip_endpoint =
        distance(chain.junctions.last().unwrap().point, candidate_endpoint) < snap_distance;
    if at_start {
        chain
            .junctions
            .extend(candidate.junctions.iter().skip(usize::from(skip_endpoint)));
    } else {
        chain.junctions.extend(
            candidate
                .junctions
                .iter()
                .rev()
                .skip(usize::from(skip_endpoint)),
        );
    }
}

fn squared_distance(left: Point, right: Point) -> i128 {
    let dx = i128::from(left.x() - right.x());
    let dy = i128::from(left.y() - right.y());
    dx * dx + dy * dy
}

fn distance(left: Point, right: Point) -> i64 {
    (squared_distance(left, right) as f64).sqrt() as i64
}

#[cfg(test)]
mod tests {
    use crate::{
        arachne::{ExtrusionJunction, ExtrusionLine},
        geometry::Point,
    };

    use super::stitch_toolpaths;

    #[test]
    fn task22o196_stitches_even_rectangle_edges_into_one_closed_line() {
        let corners = [
            Point::new(0, 0),
            Point::new(1_000, 0),
            Point::new(1_000, 1_000),
            Point::new(0, 1_000),
            Point::new(0, 0),
        ];
        let lines = corners
            .windows(2)
            .map(|edge| {
                let mut line = ExtrusionLine::new(0, false);
                line.push(ExtrusionJunction::new(edge[0], 400, 0));
                line.push(ExtrusionJunction::new(edge[1], 400, 0));
                line
            })
            .collect();
        let mut toolpaths = vec![lines];

        stitch_toolpaths(&mut toolpaths, 100, 10);

        assert_eq!(toolpaths[0].len(), 1);
        assert!(toolpaths[0][0].is_closed);
        assert_eq!(toolpaths[0][0].junctions.len(), 5);
        assert_eq!(
            toolpaths[0][0].junctions.first().unwrap().point,
            toolpaths[0][0].junctions.last().unwrap().point
        );
    }
}
