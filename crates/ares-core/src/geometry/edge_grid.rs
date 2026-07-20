pub(crate) mod raster;

use super::{ClipperError, Coord, ExPolygon, Point};
use raster::{RasterGrid, visit_line};

const BOUNDS_EPSILON: Coord = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GridEdge {
    pub(crate) contour_index: usize,
    pub(crate) segment_index: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Cell {
    begin: usize,
    end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EdgeGrid {
    bounds_min: Point,
    bounds_max: Point,
    resolution: Coord,
    rows: usize,
    cols: usize,
    contours: Vec<Vec<Point>>,
    cells: Vec<Cell>,
    cell_data: Vec<GridEdge>,
}

impl EdgeGrid {
    pub(crate) fn new(
        expolygon: &ExPolygon,
        initial_min: Point,
        initial_max: Point,
        resolution: Coord,
    ) -> Result<Self, ClipperError> {
        if resolution <= 0 || initial_min.x() > initial_max.x() || initial_min.y() > initial_max.y()
        {
            return Err(ClipperError::CoordinateOutOfRange);
        }

        let contours = std::iter::once(expolygon.contour())
            .chain(expolygon.holes())
            .filter(|polygon| !polygon.points().is_empty())
            .map(|polygon| polygon.points().to_vec())
            .collect::<Vec<_>>();
        let (mut min_x, mut min_y) = (initial_min.x(), initial_min.y());
        let (mut max_x, mut max_y) = (initial_max.x(), initial_max.y());
        for point in contours.iter().flatten() {
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
        }
        min_x = min_x
            .checked_sub(BOUNDS_EPSILON)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        min_y = min_y
            .checked_sub(BOUNDS_EPSILON)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        max_x = max_x
            .checked_add(BOUNDS_EPSILON)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        max_y = max_y
            .checked_add(BOUNDS_EPSILON)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        let bounds_min = Point::new(min_x, min_y);
        let bounds_max = Point::new(max_x, max_y);
        let cols = dimension(min_x, max_x, resolution)?;
        let rows = dimension(min_y, max_y, resolution)?;
        let cell_count = rows
            .checked_mul(cols)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(cell_count)
            .map_err(|_| ClipperError::CoordinateOutOfRange)?;
        cells.resize(cell_count, Cell::default());
        let raster_grid = RasterGrid {
            bounds_min,
            resolution,
            rows,
            cols,
        };

        for_each_edge(&contours, |edge, p1, p2| {
            visit_line(raster_grid, p1, p2, |row, col| {
                let cell = &mut cells[row * cols + col];
                cell.end += 1;
                true
            })?;
            let _ = edge;
            Ok(())
        })?;

        let mut total = 0usize;
        for cell in &mut cells {
            let count = cell.end;
            cell.begin = total;
            total = total
                .checked_add(count)
                .ok_or(ClipperError::CoordinateOutOfRange)?;
            cell.end = cell.begin;
        }
        let mut cell_data = Vec::new();
        cell_data
            .try_reserve_exact(total)
            .map_err(|_| ClipperError::CoordinateOutOfRange)?;
        cell_data.resize(
            total,
            GridEdge {
                contour_index: 0,
                segment_index: 0,
            },
        );

        for_each_edge(&contours, |edge, p1, p2| {
            visit_line(raster_grid, p1, p2, |row, col| {
                let cell = &mut cells[row * cols + col];
                cell_data[cell.end] = edge;
                cell.end += 1;
                true
            })
        })?;

        Ok(Self {
            bounds_min,
            bounds_max,
            resolution,
            rows,
            cols,
            contours,
            cells,
            cell_data,
        })
    }

    pub(crate) const fn bounds(&self) -> (Point, Point) {
        (self.bounds_min, self.bounds_max)
    }

    pub(crate) const fn resolution(&self) -> Coord {
        self.resolution
    }

    pub(crate) const fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub(crate) fn contour(&self, index: usize) -> &[Point] {
        &self.contours[index]
    }

    pub(crate) fn segment(&self, edge: GridEdge) -> (Point, Point) {
        let contour = &self.contours[edge.contour_index];
        (
            contour[edge.segment_index],
            contour[(edge.segment_index + 1) % contour.len()],
        )
    }

    pub(crate) fn visit_cells_intersecting_box<Visitor>(
        &self,
        query_min: Point,
        query_max: Point,
        mut visitor: Visitor,
    ) where
        Visitor: FnMut(usize, usize, &[GridEdge]) -> bool,
    {
        let resolution = i128::from(self.resolution);
        let min_col = (i128::from(query_min.x()) - i128::from(self.bounds_min.x())) / resolution;
        let min_row = (i128::from(query_min.y()) - i128::from(self.bounds_min.y())) / resolution;
        let max_col =
            (i128::from(query_max.x()) - i128::from(self.bounds_min.x()) - 1) / resolution;
        let max_row =
            (i128::from(query_max.y()) - i128::from(self.bounds_min.y()) - 1) / resolution;
        let start_col = min_col.max(0);
        let start_row = min_row.max(0);
        let end_col = max_col.min(self.cols as i128 - 1);
        let end_row = max_row.min(self.rows as i128 - 1);
        if start_col > end_col || start_row > end_row {
            return;
        }

        for row in start_row..=end_row {
            let row = usize::try_from(row).expect("clamped grid row must fit usize");
            if !self.visit_row(row, start_col, end_col, &mut visitor) {
                return;
            }
        }
    }

    fn visit_row<Visitor>(
        &self,
        row: usize,
        start_col: i128,
        end_col: i128,
        visitor: &mut Visitor,
    ) -> bool
    where
        Visitor: FnMut(usize, usize, &[GridEdge]) -> bool,
    {
        for col in start_col..=end_col {
            let col = usize::try_from(col).expect("clamped grid column must fit usize");
            let cell = self.cells[row * self.cols + col];
            if !visitor(row, col, &self.cell_data[cell.begin..cell.end]) {
                return false;
            }
        }
        true
    }
}

fn dimension(min: Coord, max: Coord, resolution: Coord) -> Result<usize, ClipperError> {
    let span = i128::from(max) - i128::from(min);
    let resolution = i128::from(resolution);
    usize::try_from((span + resolution - 1) / resolution)
        .map_err(|_| ClipperError::CoordinateOutOfRange)
}

fn for_each_edge(
    contours: &[Vec<Point>],
    mut visitor: impl FnMut(GridEdge, Point, Point) -> Result<(), ClipperError>,
) -> Result<(), ClipperError> {
    for (contour_index, contour) in contours.iter().enumerate() {
        for segment_index in 0..contour.len() {
            visitor(
                GridEdge {
                    contour_index,
                    segment_index,
                },
                contour[segment_index],
                contour[(segment_index + 1) % contour.len()],
            )?;
        }
    }
    Ok(())
}
