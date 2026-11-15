use std::collections::HashMap;

use crate::geometry::{Point, Polygon};

type Segment = (usize, usize);

pub(super) struct LineGrid {
    cell_size: i64,
    cells: HashMap<(i64, i64), Vec<Segment>>,
}

impl LineGrid {
    pub(super) fn new(polygons: &[Polygon], cell_size: i64) -> Self {
        let mut grid = Self {
            cell_size,
            cells: HashMap::new(),
        };
        for (polygon_index, polygon) in polygons.iter().enumerate() {
            let points = polygon.points();
            for (point_index, &start) in points.iter().enumerate() {
                grid.insert(
                    start,
                    points[(point_index + 1) % points.len()],
                    (polygon_index, point_index),
                );
            }
        }
        grid
    }

    // SquareGrid.cpp::processLineCells uses an inclusive fat-line traversal,
    // truncating division and overlapping cells between successive rows.
    fn insert(&mut self, mut start: Point, mut end: Point, segment: Segment) {
        if end.x() < start.x() {
            std::mem::swap(&mut start, &mut end);
        }
        let size = self.cell_size;
        let start_cell = (start.x() / size, start.y() / size);
        let end_cell = (end.x() / size, end.y() / size);
        let y_diff = end.y() - start.y();
        let y_direction = nonzero_sign(y_diff);
        let mut x_start = start_cell.0;
        let mut y = start_cell.1;
        while y * y_direction <= end_cell.1 * y_direction {
            let x_end = self.row_end(start, end, y, x_start);
            if self.insert_row(x_start..=x_end, y, end_cell, segment) {
                return;
            }
            x_start = x_end;
            y += y_direction;
        }
        unreachable!("source line traversal reaches its end cell");
    }

    fn row_end(&self, start: Point, end: Point, y: i64, x_start: i64) -> i64 {
        let size = self.cell_size;
        let y_diff = end.y() - start.y();
        if y_diff == 0 {
            return end.x() / size;
        }
        let direction = nonzero_sign(y_diff);
        let next_y = (y + if nonzero_sign(y) == direction || y == 0 {
            direction
        } else {
            0
        }) * size;
        let area = (end.x() - start.x()) * (next_y - start.y());
        let corresponding_x = start.x() + area / y_diff;
        let last = (corresponding_x + i64::from(corresponding_x < 0 && area % y_diff != 0)) / size;
        if last < start.x() / size {
            x_start
        } else {
            last
        }
    }

    fn insert_row(
        &mut self,
        xs: std::ops::RangeInclusive<i64>,
        y: i64,
        end_cell: (i64, i64),
        segment: Segment,
    ) -> bool {
        for x in xs {
            self.cells.entry((x, y)).or_default().push(segment);
            if (x, y) == end_cell {
                return true;
            }
        }
        false
    }

    pub(super) fn nearby(&self, point: Point, radius: i64) -> Vec<Segment> {
        let size = self.cell_size;
        let mut nearby = Vec::new();
        for y in (point.y() - radius) / size..=(point.y() + radius) / size {
            for x in (point.x() - radius) / size..=(point.x() + radius) / size {
                // Native libstdc++ emplace puts equivalent keys first.
                // Retain repeated segments from different queried cells.
                nearby.extend(self.cells.get(&(x, y)).into_iter().flatten().rev().copied());
            }
        }
        nearby
    }
}

fn nonzero_sign(value: i64) -> i64 {
    if value < 0 { -1 } else { 1 }
}
