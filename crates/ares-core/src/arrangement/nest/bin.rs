//! The bin queries of `geometry_traits.hpp` and `_NofitPolyPlacer::overfit` (`nfpplacer.hpp:488-540`)
//! for the three bed shapes.

use super::Bin;
use super::geometry::{NBox, shape_box, shape_within, sub};
use super::item::translated;
use crate::geometry::{ExPolygon, Point};

impl Bin {
    /// `sl::boundingBox(bin)`: a box itself, a circle's square, a polygon's (undefined) envelope.
    pub(crate) fn bounding_box(&self) -> NBox {
        match self {
            Bin::Box(bin) => *bin,
            Bin::Circle { center, radius } => NBox::new(
                Point::new(
                    (center.x() as f64 - radius) as i64,
                    (center.y() as f64 - radius) as i64,
                ),
                Point::new(
                    (center.x() as f64 + radius) as i64,
                    (center.y() as f64 + radius) as i64,
                ),
            ),
            Bin::Polygon(polygon) => shape_box(polygon),
        }
    }

    pub(crate) fn area(&self) -> f64 {
        match self {
            Bin::Box(bin) => bin.area(),
            Bin::Circle { radius, .. } => std::f64::consts::PI * radius * radius,
            Bin::Polygon(polygon) => super::geometry::area(polygon),
        }
    }

    /// `overfit(chull, bin)`: positive when the shape does not fit.
    pub(crate) fn overfit(&self, shape: &ExPolygon) -> f64 {
        match self {
            Bin::Box(bin) => overfit_box(shape_box(shape), *bin),
            Bin::Circle { radius, .. } => super::bounding_circle(shape).1 - radius,
            Bin::Polygon(polygon) => {
                let (bbch, bbin) = (shape_box(shape), shape_box(polygon));
                let moved = translated(shape, sub(bbin.center(), bbch.center()));
                if shape_within(&moved, polygon) {
                    -1.0
                } else {
                    1.0
                }
            }
        }
    }

    /// `sl::isInside(shape, bin)`.
    pub(crate) fn contains(&self, shape: &ExPolygon) -> bool {
        match self {
            Bin::Box(bin) => {
                let bb = shape_box(shape);
                bb.min.x() >= bin.min.x()
                    && bb.max.x() <= bin.max.x()
                    && bb.min.y() >= bin.min.y()
                    && bb.max.y() <= bin.max.y()
            }
            Bin::Circle { center, radius } => shape.contour().points().iter().all(|point| {
                let (dx, dy) = (
                    center.x() as f64 - point.x() as f64,
                    center.y() as f64 - point.y() as f64,
                );
                dx * dx + dy * dy < radius * radius
            }),
            Bin::Polygon(polygon) => shape_within(shape, polygon),
        }
    }
}

/// `overfit(Box, Box)`: how much wider and taller than the bin.
pub(crate) fn overfit_box(bb: NBox, bin: NBox) -> f64 {
    let (wdiff, hdiff) = (bb.width() - bin.width(), bb.height() - bin.height());
    (if wdiff > 0 { wdiff as f64 } else { 0.0 }) + (if hdiff > 0 { hdiff as f64 } else { 0.0 })
}
