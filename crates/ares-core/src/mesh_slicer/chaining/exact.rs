use std::collections::{BTreeMap, BTreeSet};

use crate::geometry::Polygon;

use super::{EndpointReference, OpenPolyline, signed_area};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum EndpointSide {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EndpointRecord {
    polyline_index: usize,
    side: EndpointSide,
}

struct ExactIndex {
    by_key: BTreeMap<i64, BTreeSet<EndpointRecord>>,
    try_connect_reversed: bool,
}

impl ExactIndex {
    fn new(open_polylines: &[OpenPolyline], try_connect_reversed: bool) -> Self {
        let mut index = Self {
            by_key: BTreeMap::new(),
            try_connect_reversed,
        };
        for (polyline_index, polyline) in open_polylines.iter().enumerate() {
            if !polyline.consumed {
                index.insert_polyline(polyline_index, polyline);
            }
        }
        index
    }

    fn insert_polyline(&mut self, polyline_index: usize, polyline: &OpenPolyline) {
        self.insert(polyline_index, EndpointSide::Start, polyline.start);
        if self.try_connect_reversed {
            self.insert(polyline_index, EndpointSide::End, polyline.end);
        }
    }

    fn first_other(
        &self,
        reference: EndpointReference,
        active_polyline: usize,
    ) -> Option<EndpointRecord> {
        self.by_key
            .get(&reference_key(reference))?
            .iter()
            .copied()
            .find(|record| record.polyline_index != active_polyline)
    }

    fn remove_polyline(
        &mut self,
        polyline_index: usize,
        start: EndpointReference,
        end: EndpointReference,
    ) {
        self.remove(polyline_index, EndpointSide::Start, start);
        if self.try_connect_reversed {
            self.remove(polyline_index, EndpointSide::End, end);
        }
    }

    fn move_end(
        &mut self,
        polyline_index: usize,
        old_end: EndpointReference,
        new_end: EndpointReference,
    ) {
        self.remove(polyline_index, EndpointSide::End, old_end);
        self.insert(polyline_index, EndpointSide::End, new_end);
    }

    fn insert(&mut self, polyline_index: usize, side: EndpointSide, reference: EndpointReference) {
        self.by_key
            .entry(reference_key(reference))
            .or_default()
            .insert(EndpointRecord {
                polyline_index,
                side,
            });
    }

    fn remove(&mut self, polyline_index: usize, side: EndpointSide, reference: EndpointReference) {
        let key = reference_key(reference);
        let records = self.by_key.get_mut(&key).unwrap();
        assert!(records.remove(&EndpointRecord {
            polyline_index,
            side,
        }));
        if records.is_empty() {
            self.by_key.remove(&key);
        }
    }
}

fn reference_key(reference: EndpointReference) -> i64 {
    match reference {
        EndpointReference::Vertex(id) => i64::from(id),
        EndpointReference::Edge(id) => -i64::from(id),
    }
}

fn sorted_exact_indices(open_polylines: &[OpenPolyline]) -> Vec<usize> {
    let mut indices = open_polylines
        .iter()
        .enumerate()
        .filter_map(|(index, polyline)| (!polyline.consumed).then_some(index))
        .collect::<Vec<_>>();
    indices.sort_by(|&left, &right| {
        open_polylines[right]
            .length
            .total_cmp(&open_polylines[left].length)
            .then_with(|| left.cmp(&right))
    });
    indices
}

pub(super) fn chain_open_polylines_exact(
    open_polylines: &mut [OpenPolyline],
    polygons: &mut Vec<Polygon>,
    try_connect_reversed: bool,
) {
    let sorted_indices = sorted_exact_indices(open_polylines);
    let mut index = ExactIndex::new(open_polylines, try_connect_reversed);

    for seed_index in sorted_indices {
        if open_polylines[seed_index].consumed {
            continue;
        }
        open_polylines[seed_index].consumed = true;

        loop {
            let old_end = open_polylines[seed_index].end;
            let Some(record) = index.first_other(old_end, seed_index) else {
                open_polylines[seed_index].consumed = false;
                break;
            };
            let candidate_index = record.polyline_index;
            let candidate_start = open_polylines[candidate_index].start;
            let candidate_end = open_polylines[candidate_index].end;
            index.remove_polyline(candidate_index, candidate_start, candidate_end);

            let opposite_end = match record.side {
                EndpointSide::Start => candidate_end,
                EndpointSide::End => candidate_start,
            };
            let candidate_points = std::mem::take(&mut open_polylines[candidate_index].points);
            let candidate_length =
                std::mem::replace(&mut open_polylines[candidate_index].length, 0.0);
            open_polylines[candidate_index].consumed = true;

            let seed = &mut open_polylines[seed_index];
            match record.side {
                EndpointSide::Start => seed.points.extend(candidate_points.into_iter().skip(1)),
                EndpointSide::End => seed
                    .points
                    .extend(candidate_points.into_iter().rev().skip(1)),
            }
            seed.length += candidate_length;
            if try_connect_reversed {
                index.move_end(seed_index, old_end, opposite_end);
                seed.end = opposite_end;
            }

            if seed.start == seed.end {
                index.remove_polyline(seed_index, seed.start, seed.end);
                close_seed(seed, polygons, try_connect_reversed);
                break;
            }
        }
    }
}

fn close_seed(seed: &mut OpenPolyline, polygons: &mut Vec<Polygon>, try_connect_reversed: bool) {
    seed.points.pop();
    if seed.points.len() < 3 {
        seed.points.clear();
        return;
    }
    if try_connect_reversed && signed_area(&seed.points) < 0.0 {
        seed.points.reverse();
    }
    polygons.push(Polygon::new(std::mem::take(&mut seed.points)));
}

#[cfg(test)]
mod tests;
