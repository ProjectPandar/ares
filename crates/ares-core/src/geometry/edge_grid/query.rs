use super::{Cell, EdgeGrid, GridEdge, RasterGrid, visit_line};
use crate::geometry::{ClipperError, Coord, Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClosestPointResult {
    pub(crate) contour_index: usize,
    pub(crate) segment_index: usize,
    pub(crate) distance: f64,
    pub(crate) t: f64,
}

impl EdgeGrid {
    #[expect(
        clippy::excessive_nesting,
        reason = "the source query preserves row, cell, edge, and vertex-ownership traversal order"
    )]
    pub(crate) fn closest_point_signed_distance(
        &self,
        point: Point,
        search_radius: Coord,
    ) -> Result<Option<ClosestPointResult>, ClipperError> {
        let resolution = i128::from(self.resolution);
        let radius = i128::from(search_radius);
        let point_col = i128::from(point.x()) - i128::from(self.bounds_min.x());
        let point_row = i128::from(point.y()) - i128::from(self.bounds_min.y());
        let max_col = point_col + radius;
        let max_row = point_row + radius;
        if max_col < 0 || max_row < 0 {
            return Ok(None);
        }

        let start_col = (point_col - radius).max(0) / resolution;
        let start_row = (point_row - radius).max(0) / resolution;
        let end_col = (max_col / resolution).min(self.cols as i128 - 1);
        let end_row = (max_row / resolution).min(self.rows as i128 - 1);
        if start_col > end_col || start_row > end_row {
            return Ok(None);
        }

        let mut minimum_distance = search_radius as f64;
        let mut result = None;
        for row in start_row..=end_row {
            let row = usize::try_from(row).map_err(|_| ClipperError::CoordinateOutOfRange)?;
            for col in start_col..=end_col {
                let col = usize::try_from(col).map_err(|_| ClipperError::CoordinateOutOfRange)?;
                let cell = self.cells[row * self.cols + col];
                for &edge in &self.cell_data[cell.begin..cell.end] {
                    let (segment_start, segment_end) = self.segment(edge);
                    let segment = vector(segment_start, segment_end);
                    let from_start = vector(segment_start, point);
                    let point_projection = dot(segment, from_start)?;
                    let segment_length_squared = dot(segment, segment)?;

                    if point_projection < 0 {
                        let distance = (dot(from_start, from_start)? as f64).sqrt();
                        if distance < minimum_distance {
                            let contour = &self.contours[edge.contour_index];
                            let previous =
                                contour[(edge.segment_index + contour.len() - 1) % contour.len()];
                            let previous_segment = vector(previous, segment_start);
                            if dot(previous_segment, from_start)? > 0 {
                                let determinant = cross(previous_segment, segment)?;
                                debug_assert_ne!(determinant, 0);
                                minimum_distance = distance;
                                result = Some(ClosestPointResult {
                                    contour_index: edge.contour_index,
                                    segment_index: edge.segment_index,
                                    distance: if determinant > 0 { distance } else { -distance },
                                    t: 0.0,
                                });
                            }
                        }
                    } else if point_projection <= segment_length_squared {
                        let distance_numerator = cross(from_start, segment)?;
                        let distance =
                            distance_numerator as f64 / (segment_length_squared as f64).sqrt();
                        let distance_abs = distance.abs();
                        if distance_abs < minimum_distance {
                            minimum_distance = distance_abs;
                            result = Some(ClosestPointResult {
                                contour_index: edge.contour_index,
                                segment_index: edge.segment_index,
                                distance,
                                t: point_projection as f64 / segment_length_squared as f64,
                            });
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    pub(crate) fn visit_cells_intersecting_line<Visitor>(
        &self,
        p1: Point,
        p2: Point,
        mut visitor: Visitor,
    ) -> Result<(), ClipperError>
    where
        Visitor: FnMut(usize, usize, &[GridEdge]) -> bool,
    {
        let resolution = i128::from(self.resolution);
        for point in [p1, p2] {
            debug_assert!(
                point.x() >= self.bounds_min.x()
                    && point.x() <= self.bounds_max.x()
                    && point.y() >= self.bounds_min.y()
                    && point.y() <= self.bounds_max.y()
            );
            let x = i128::from(point.x()) - i128::from(self.bounds_min.x());
            let y = i128::from(point.y()) - i128::from(self.bounds_min.y());
            debug_assert!(x >= 0 && x < self.cols as i128 * resolution);
            debug_assert!(y >= 0 && y < self.rows as i128 * resolution);
        }

        visit_line(
            RasterGrid {
                bounds_min: self.bounds_min,
                resolution: self.resolution,
                rows: self.rows,
                cols: self.cols,
            },
            p1,
            p2,
            |row, col| {
                let Cell { begin, end } = self.cells[row * self.cols + col];
                visitor(row, col, &self.cell_data[begin..end])
            },
        )
    }
}

fn vector(from: Point, to: Point) -> (i128, i128) {
    (
        i128::from(to.x()) - i128::from(from.x()),
        i128::from(to.y()) - i128::from(from.y()),
    )
}

fn dot(left: (i128, i128), right: (i128, i128)) -> Result<i128, ClipperError> {
    left.0
        .checked_mul(right.0)
        .and_then(|x| left.1.checked_mul(right.1).and_then(|y| x.checked_add(y)))
        .ok_or(ClipperError::CoordinateOutOfRange)
}

fn cross(left: (i128, i128), right: (i128, i128)) -> Result<i128, ClipperError> {
    left.0
        .checked_mul(right.1)
        .and_then(|x| left.1.checked_mul(right.0).and_then(|y| x.checked_sub(y)))
        .ok_or(ClipperError::CoordinateOutOfRange)
}
