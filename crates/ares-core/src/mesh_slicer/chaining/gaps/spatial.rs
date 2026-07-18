use std::collections::BTreeMap;

use crate::geometry::{Coord, Point};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum EndpointSide {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct EndpointKey {
    pub(super) original_index: usize,
    pub(super) side: EndpointSide,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Cell {
    x: i128,
    y: i128,
}

#[derive(Clone, Copy)]
struct Entry {
    key: EndpointKey,
    point: Point,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Nearest {
    pub(super) key: EndpointKey,
    pub(super) distance_squared: u128,
}

pub(super) struct RadiusGrid {
    radius: Coord,
    radius_squared: u128,
    cells: BTreeMap<Cell, Vec<Entry>>,
    locations: BTreeMap<EndpointKey, Cell>,
}

impl RadiusGrid {
    pub(super) fn new(radius: Coord) -> Self {
        let radius_squared = (radius as u128).pow(2);
        Self {
            radius,
            radius_squared,
            cells: BTreeMap::new(),
            locations: BTreeMap::new(),
        }
    }

    pub(super) fn insert(&mut self, key: EndpointKey, point: Point) {
        let cell = cell(point, self.radius);
        let previous = self.locations.insert(key, cell);
        assert!(previous.is_none());
        let entries = self.cells.entry(cell).or_default();
        let position = entries
            .binary_search_by_key(&key, |entry| entry.key)
            .unwrap_err();
        entries.insert(position, Entry { key, point });
    }

    pub(super) fn remove(&mut self, key: EndpointKey) -> bool {
        let Some(cell) = self.locations.remove(&key) else {
            return false;
        };
        let entries = self.cells.get_mut(&cell).unwrap();
        let position = entries
            .binary_search_by_key(&key, |entry| entry.key)
            .unwrap();
        entries.remove(position);
        if entries.is_empty() {
            self.cells.remove(&cell);
        }
        true
    }

    pub(super) fn find(&self, query: Point, is_active: impl Fn(usize) -> bool) -> Option<Nearest> {
        let center = cell(query, self.radius);
        let mut best = None::<(u128, EndpointKey)>;
        for y_offset in -1_i128..=1 {
            for x_offset in -1_i128..=1 {
                let neighbor = Cell {
                    x: center.x + x_offset,
                    y: center.y + y_offset,
                };
                self.find_in_cell(neighbor, query, &is_active, &mut best);
            }
        }
        best.map(|(distance_squared, key)| Nearest {
            key,
            distance_squared,
        })
    }

    fn find_in_cell(
        &self,
        cell: Cell,
        query: Point,
        is_active: &impl Fn(usize) -> bool,
        best: &mut Option<(u128, EndpointKey)>,
    ) {
        let Some(entries) = self.cells.get(&cell) else {
            return;
        };
        for entry in entries {
            if !is_active(entry.key.original_index) {
                continue;
            }
            let Some(distance_squared) = distance_squared_inside_with_square(
                query,
                entry.point,
                self.radius,
                self.radius_squared,
            ) else {
                continue;
            };
            let rank = (distance_squared, entry.key);
            if best.is_none_or(|current| rank < current) {
                *best = Some(rank);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.locations.len()
    }
}

pub(super) fn distance_squared_inside(a: Point, b: Point, radius: Coord) -> Option<u128> {
    distance_squared_inside_with_square(a, b, radius, (radius as u128).pow(2))
}

fn distance_squared_inside_with_square(
    a: Point,
    b: Point,
    radius: Coord,
    radius_squared: u128,
) -> Option<u128> {
    let radius = radius as u128;
    let x_difference = (i128::from(a.x()) - i128::from(b.x())).unsigned_abs();
    let y_difference = (i128::from(a.y()) - i128::from(b.y())).unsigned_abs();
    if x_difference >= radius || y_difference >= radius {
        return None;
    }
    let distance_squared = x_difference.pow(2) + y_difference.pow(2);
    (distance_squared < radius_squared).then_some(distance_squared)
}

fn cell(point: Point, radius: Coord) -> Cell {
    let radius = i128::from(radius);
    Cell {
        x: i128::from(point.x()).div_euclid(radius),
        y: i128::from(point.y()).div_euclid(radius),
    }
}

#[cfg(test)]
mod tests;
