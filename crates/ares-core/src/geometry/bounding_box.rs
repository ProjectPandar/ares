// Ports the fixed-coordinate `BoundingBox`/`chain_expolygons` seam from
// OrcaSlicer `BoundingBox.hpp` and `ShortestPath.cpp`.

use super::{ExPolygon, Point, Polygon, chain_points};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    pub(crate) fn from_points(mut points: impl Iterator<Item = Point>) -> Option<Self> {
        let first = points.next()?;
        let mut bounds = Self::new(first, first);
        for point in points {
            bounds.merge_point(point);
        }
        Some(bounds)
    }

    pub(crate) fn contains(self, point: Point) -> bool {
        point.x() >= self.min.x()
            && point.y() >= self.min.y()
            && point.x() <= self.max.x()
            && point.y() <= self.max.y()
    }

    pub(crate) fn contains_f64(self, point: [f64; 2]) -> bool {
        point[0] >= self.min.x() as f64
            && point[1] >= self.min.y() as f64
            && point[0] <= self.max.x() as f64
            && point[1] <= self.max.y() as f64
    }

    pub(crate) fn overlap(self, other: Self) -> bool {
        !(self.max.x() < other.min.x()
            || self.min.x() > other.max.x()
            || self.max.y() < other.min.y()
            || self.min.y() > other.max.y())
    }

    pub(crate) fn size(self) -> [f64; 2] {
        [
            (self.max.x() - self.min.x()) as f64,
            (self.max.y() - self.min.y()) as f64,
        ]
    }

    pub const fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    pub(crate) fn from_polygon(polygon: &Polygon) -> Option<Self> {
        let first = *polygon.points().first()?;
        let (min_x, min_y, max_x, max_y) = polygon.points().iter().skip(1).fold(
            (first.x(), first.y(), first.x(), first.y()),
            |(min_x, min_y, max_x, max_y), point| {
                (
                    min_x.min(point.x()),
                    min_y.min(point.y()),
                    max_x.max(point.x()),
                    max_y.max(point.y()),
                )
            },
        );
        Some(Self {
            min: Point::new(min_x, min_y),
            max: Point::new(max_x, max_y),
        })
    }

    pub(crate) fn from_polygons(polygons: &[Polygon]) -> Option<Self> {
        let mut bounds = Self::from_polygon(polygons.first()?)?;
        for polygon in &polygons[1..] {
            let next = Self::from_polygon(polygon)?;
            bounds.min = Point::new(
                bounds.min.x().min(next.min.x()),
                bounds.min.y().min(next.min.y()),
            );
            bounds.max = Point::new(
                bounds.max.x().max(next.max.x()),
                bounds.max.y().max(next.max.y()),
            );
        }
        Some(bounds)
    }

    pub fn from_expolygon(expolygon: &ExPolygon) -> Option<Self> {
        Self::from_polygon(expolygon.contour())
    }

    pub(crate) fn from_expolygons(expolygons: &[ExPolygon]) -> Option<Self> {
        let mut bounds = Self::from_expolygon(expolygons.first()?)?;
        for expolygon in &expolygons[1..] {
            let next = Self::from_expolygon(expolygon)?;
            bounds.min = Point::new(
                bounds.min.x().min(next.min.x()),
                bounds.min.y().min(next.min.y()),
            );
            bounds.max = Point::new(
                bounds.max.x().max(next.max.x()),
                bounds.max.y().max(next.max.y()),
            );
        }
        Some(bounds)
    }

    pub const fn min(self) -> Point {
        self.min
    }

    pub const fn max(self) -> Point {
        self.max
    }

    /// `BoundingBox::merge` — grow to contain `point`.
    pub(crate) fn merge_point(&mut self, point: Point) {
        self.min = Point::new(self.min.x().min(point.x()), self.min.y().min(point.y()));
        self.max = Point::new(self.max.x().max(point.x()), self.max.y().max(point.y()));
    }

    /// `BoundingBox::inflated` — grow by `delta` on all sides (returns copy).
    pub(crate) fn inflated(mut self, delta: i64) -> Self {
        self.offset(delta);
        self
    }

    pub(crate) fn offset(&mut self, delta: i64) {
        self.min = Point::new(self.min.x() - delta, self.min.y() - delta);
        self.max = Point::new(self.max.x() + delta, self.max.y() + delta);
    }

    pub fn center(self) -> Point {
        Point::new(
            midpoint(self.min.x(), self.max.x()),
            midpoint(self.min.y(), self.max.y()),
        )
    }
}

fn midpoint(left: i64, right: i64) -> i64 {
    ((i128::from(left) + i128::from(right)) / 2) as i64
}

pub(crate) fn chain_expolygons_order(expolygons: &[ExPolygon]) -> Vec<usize> {
    let centers = expolygons
        .iter()
        .map(|expolygon| {
            BoundingBox::from_expolygon(expolygon)
                .expect("a sliced ExPolygon contour must be nonempty")
                .center()
        })
        .collect::<Vec<_>>();
    chain_points(&centers)
}

pub(crate) fn chain_expolygons(expolygons: Vec<ExPolygon>) -> Vec<ExPolygon> {
    let order = chain_expolygons_order(&expolygons);
    let mut source = expolygons.into_iter().map(Some).collect::<Vec<_>>();
    order
        .into_iter()
        .map(|index| {
            source[index]
                .take()
                .expect("chain order must contain each input once")
        })
        .collect()
}
