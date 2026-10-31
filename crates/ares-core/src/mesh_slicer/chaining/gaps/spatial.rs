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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Nearest {
    pub(super) key: EndpointKey,
    pub(super) distance_squared: f64,
}

/// Port of upstream `ClosestPointInRadiusLookup` (`Point.hpp:506`). The grid
/// resolution is `2*radius + 4` rounded up to a power of two (`m_grid_log2`);
/// insertion buckets points by `coord >> log2`. `find` rounds the query to the
/// nearest cell corner and scans the four `{-1,0} x {-1,0}` cells around it,
/// returning the closest candidate whose squared distance is below
/// `search_radius^2` (double comparison, first-found wins ties).
pub(super) struct RadiusGrid {
    radius_squared: f64,
    log2: u32,
    cells: BTreeMap<Cell, Vec<Entry>>,
    locations: BTreeMap<EndpointKey, Cell>,
}

impl RadiusGrid {
    pub(super) fn new(radius: Coord) -> Self {
        // `m_grid_resolution = 2*radius + 4`, then rounded up to a power of two.
        let mut grid_resolution = 2 * radius + 4;
        let mut log2 = 0_u32;
        if grid_resolution > 32767 {
            grid_resolution >>= 16;
            log2 += 16;
        }
        if grid_resolution > 127 {
            grid_resolution >>= 8;
            log2 += 8;
        }
        if grid_resolution > 7 {
            grid_resolution >>= 4;
            log2 += 4;
        }
        if grid_resolution > 1 {
            grid_resolution >>= 2;
            log2 += 2;
        }
        if grid_resolution > 0 {
            log2 += 1;
        }
        Self {
            radius_squared: (radius as f64).powi(2),
            log2,
            cells: BTreeMap::new(),
            locations: BTreeMap::new(),
        }
    }

    pub(super) fn insert(&mut self, key: EndpointKey, point: Point) {
        let cell = self.cell_of(point);
        let previous = self.locations.insert(key, cell);
        assert!(previous.is_none());
        // Upstream `unordered_multimap::equal_range` iterates a cell's entries
        // in insertion order; preserve that so equidistant `d2 < dist_min`
        // ties resolve to the same endpoint the oracle returns.
        self.cells
            .entry(cell)
            .or_default()
            .push(Entry { key, point });
    }

    pub(super) fn remove(&mut self, key: EndpointKey) -> bool {
        let Some(cell) = self.locations.remove(&key) else {
            return false;
        };
        let entries = self.cells.get_mut(&cell).unwrap();
        // Upstream `m_map.erase(it)` removes in place; keep insertion order.
        let position = entries.iter().position(|entry| entry.key == key).unwrap();
        entries.remove(position);
        if entries.is_empty() {
            self.cells.remove(&cell);
        }
        true
    }

    pub(super) fn find(&self, query: Point, is_active: impl Fn(usize) -> bool) -> Option<Nearest> {
        // Round the query to the closest grid cell corner: `(pt + res/2) >> log2`.
        let corner = self.corner_of(query);
        let mut best: Option<(f64, EndpointKey)> = None;
        // Four neighbor cells: offsets -1 and 0 on each axis.
        for neighbor_y in -1_i128..1 {
            for neighbor_x in -1_i128..1 {
                let cell = Cell {
                    x: corner.x + neighbor_x,
                    y: corner.y + neighbor_y,
                };
                self.find_in_cell(cell, query, &is_active, &mut best);
            }
        }
        best.and_then(|(distance_squared, key)| {
            (distance_squared < self.radius_squared).then_some(Nearest {
                key,
                distance_squared,
            })
        })
    }

    fn find_in_cell(
        &self,
        cell: Cell,
        query: Point,
        is_active: &impl Fn(usize) -> bool,
        best: &mut Option<(f64, EndpointKey)>,
    ) {
        let Some(entries) = self.cells.get(&cell) else {
            return;
        };
        for entry in entries {
            if !is_active(entry.key.original_index) {
                continue;
            }
            let distance_squared = squared_distance(query, entry.point);
            if best.is_none_or(|current| distance_squared < current.0) {
                *best = Some((distance_squared, entry.key));
            }
        }
    }

    /// `pt >> m_grid_log2` (floor division by `1 << log2`).
    fn cell_of(&self, point: Point) -> Cell {
        Cell {
            x: i128::from(point.x()) >> self.log2,
            y: i128::from(point.y()) >> self.log2,
        }
    }

    /// `(pt + res/2) >> m_grid_log2`: round to nearest cell corner.
    fn corner_of(&self, point: Point) -> Cell {
        let half = 1_i128 << (self.log2 - 1);
        Cell {
            x: (i128::from(point.x()) + half) >> self.log2,
            y: (i128::from(point.y()) + half) >> self.log2,
        }
    }
}

/// `(a - b).cast<double>().squaredNorm()` — squared distance in f64.
fn squared_distance(a: Point, b: Point) -> f64 {
    let dx = a.x() as f64 - b.x() as f64;
    let dy = a.y() as f64 - b.y() as f64;
    dx * dx + dy * dy
}

/// `(a - b).squaredNorm()` bounded to `radius` — used for the loop-closing
/// distance check. Returns the f64 squared distance when it is below `radius^2`.
pub(super) fn distance_squared_inside(a: Point, b: Point, radius: Coord) -> Option<f64> {
    let distance_squared = squared_distance(a, b);
    (distance_squared < (radius as f64).powi(2)).then_some(distance_squared)
}
