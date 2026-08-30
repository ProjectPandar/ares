// Ports the fixed-coordinate `BoundingBox`/`chain_expolygons` seam from
// OrcaSlicer `BoundingBox.hpp` and `ShortestPath.cpp`.

use super::{ExPolygon, Point, Polygon, chain_points};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    pub(crate) const fn new(min: Point, max: Point) -> Self {
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

    pub(crate) fn from_expolygon(expolygon: &ExPolygon) -> Option<Self> {
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

    pub(crate) const fn min(self) -> Point {
        self.min
    }

    pub(crate) const fn max(self) -> Point {
        self.max
    }

    pub(crate) fn offset(&mut self, delta: i64) {
        self.min = Point::new(self.min.x() - delta, self.min.y() - delta);
        self.max = Point::new(self.max.x() + delta, self.max.y() + delta);
    }

    pub(crate) fn center(self) -> Point {
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
