use super::{Point, ThickLine};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Polyline {
    points: Vec<Point>,
}

impl Polyline {
    pub(crate) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }

    pub(crate) fn points(&self) -> &[Point] {
        &self.points
    }

    pub(crate) fn points_mut(&mut self) -> &mut Vec<Point> {
        &mut self.points
    }

    pub(crate) fn into_points(self) -> Vec<Point> {
        self.points
    }

    pub(crate) fn front(&self) -> Option<Point> {
        self.points.first().copied()
    }

    pub(crate) fn back(&self) -> Option<Point> {
        self.points.last().copied()
    }

    pub(crate) fn reverse(&mut self) {
        self.points.reverse();
    }

    pub(crate) fn append_points(&mut self, points: impl IntoIterator<Item = Point>) {
        self.points.extend(points);
    }
    pub(crate) fn is_valid(&self) -> bool {
        self.points.len() >= 2
    }

    /// `Polyline::clip_end` (`Polyline.cpp:50-68`) — removes `distance` from the
    /// polyline tail, walking back whole segments and interpolating the final
    /// partial segment with `.cast<coord_t>()` (truncation). Plain `Polyline`
    /// carries no `fitting_result`, so the arc-clip bookkeeping is omitted.
    pub(crate) fn clip_end(&mut self, mut distance: f64) {
        while distance > 0.0 {
            let Some(last_point) = self.points.last().copied() else {
                return;
            };
            self.points.pop();
            let Some(new_last) = self.points.last().copied() else {
                return;
            };
            let vx = (new_last.x() - last_point.x()) as f64;
            let vy = (new_last.y() - last_point.y()) as f64;
            let lsqr = vx * vx + vy * vy;
            if lsqr > distance * distance {
                let t = distance / lsqr.sqrt();
                self.points.push(Point::new(
                    (last_point.x() as f64 + vx * t) as i64,
                    (last_point.y() as f64 + vy * t) as i64,
                ));
                break;
            }
            distance -= lsqr.sqrt();
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ThickPolyline {
    pub(crate) points: Vec<Point>,
    pub(crate) width: Vec<f64>,
    pub(crate) endpoints: (bool, bool),
}

impl ThickPolyline {
    pub(crate) fn length(&self) -> f64 {
        self.points
            .windows(2)
            .map(|points| super::Line::new(points[0], points[1]).length())
            .sum()
    }

    pub(crate) fn reverse(&mut self) {
        self.points.reverse();
        self.width.reverse();
        self.endpoints = (self.endpoints.1, self.endpoints.0);
    }

    pub(crate) fn clear(&mut self) {
        self.points.clear();
        self.width.clear();
    }

    pub(crate) fn thicklines(&self) -> Vec<ThickLine> {
        self.points
            .windows(2)
            .enumerate()
            .map(|(index, points)| {
                ThickLine::with_widths(
                    points[0],
                    points[1],
                    self.width[2 * index],
                    self.width[2 * index + 1],
                )
            })
            .collect()
    }

    pub(crate) fn start_at_index(&mut self, index: usize) {
        if index != 0
            && index + 1 != self.points.len()
            && self.points.first() == self.points.last()
            && self.width.first() == self.width.last()
        {
            self.points.pop();
            self.points.rotate_left(index);
            self.width.rotate_left(2 * index);
            self.points.push(self.points[0]);
        }
    }

    pub(crate) fn clip_end(&mut self, mut distance: f64) {
        let mut last_point_inserted = false;
        let mut remove_after_index = self.points.len();
        for point_index in (1..self.points.len()).rev() {
            let last = self.points[point_index];
            let previous = self.points[point_index - 1];
            let line_length = super::Line::new(previous, last).length();
            if line_length > distance {
                let dx = (last.x() - previous.x()) as f64 / line_length;
                let dy = (last.y() - previous.y()) as f64 / line_length;
                self.points.insert(
                    point_index,
                    Point::new(
                        (last.x() as f64 - dx * distance) as i64,
                        (last.y() as f64 - dy * distance) as i64,
                    ),
                );
                remove_after_index = point_index + 1;
                last_point_inserted = true;
                break;
            }
            if (line_length - distance).abs() < f64::EPSILON {
                remove_after_index = point_index;
                last_point_inserted = true;
                break;
            }
            distance -= line_length;
        }
        if last_point_inserted {
            self.points.truncate(remove_after_index);
        } else {
            self.points.clear();
        }
    }
}

pub(crate) fn to_thick_polylines(polylines: Vec<Polyline>, width: f64) -> Vec<ThickPolyline> {
    polylines
        .into_iter()
        .map(|polyline| {
            let points = polyline.into_points();
            ThickPolyline {
                width: vec![width; (points.len() - 1) * 2],
                points,
                endpoints: (false, false),
            }
        })
        .collect()
}
