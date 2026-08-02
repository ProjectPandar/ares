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

    pub(crate) fn is_valid(&self) -> bool {
        self.points.len() >= 2
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ThickPolyline {
    pub(crate) points: Vec<Point>,
    pub(crate) width: Vec<f64>,
    pub(crate) endpoints: (bool, bool),
}

impl ThickPolyline {
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
        assert!(index < self.points.len());
        assert_eq!(self.points.first(), self.points.last());
        assert_eq!(self.width.first(), self.width.last());
        if index != 0 && index + 1 != self.points.len() {
            self.points.pop();
            assert_eq!(self.points.len() * 2, self.width.len());
            self.points.rotate_left(index);
            self.width.rotate_left(2 * index);
            self.points.push(self.points[0]);
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
