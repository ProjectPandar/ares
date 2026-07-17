mod coord;
mod polygon;

pub(crate) use coord::{Coord, CoordinateScale, Point};
pub(crate) use polygon::Polygon;

const _: usize = std::mem::size_of::<Coord>();
const _: fn(&crate::Point2dList) -> CoordinateScale = CoordinateScale::from_printable_area;
const _: fn(CoordinateScale) -> f64 = CoordinateScale::factor;
const _: fn(CoordinateScale, f64) -> Option<Coord> = CoordinateScale::checked_scale;
const _: fn(CoordinateScale, Coord) -> f64 = CoordinateScale::unscale;
const _: fn(Coord, Coord) -> Point = Point::new;
const _: fn(Point) -> Coord = Point::x;
const _: fn(Point) -> Coord = Point::y;

#[cfg(test)]
mod tests;
