use super::Point;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Polygon {
    points: Vec<Point>,
}

impl Polygon {
    pub(crate) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }

    pub(crate) fn points(&self) -> &[Point] {
        &self.points
    }
}
