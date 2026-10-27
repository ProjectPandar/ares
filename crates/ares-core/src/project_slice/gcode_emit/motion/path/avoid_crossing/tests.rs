mod boundary_parity;
mod empty_boundary;
mod integer_nudge;

use super::Request;
use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    options::FloatOrPercent,
    project_slice::{
        gcode_emit::motion::{
            LayerGeometry, arc::Point as MotionPoint, state::AvoidCrossingGeometry,
        },
        region_slices::RegionSurface,
    },
};

fn geometry() -> LayerGeometry<'static> {
    let surface = RegionSurface::internal(ExPolygon::new(
        Polygon::new(vec![
            Point::new(-5_000_000, -5_000_000),
            Point::new(5_000_000, -5_000_000),
            Point::new(5_000_000, 5_000_000),
            Point::new(-5_000_000, 5_000_000),
        ]),
        Vec::new(),
    ));
    LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: Box::leak(vec![&*Box::leak(Box::new(surface))].into_boxed_slice()),
        support_islands: None,
        scale: CoordinateScale::Normal,
        previous_curled: &[],
        previous_layer_boundary: None,
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.42,
            layer_slices: &[],
            perimeter_spacing: 0.0,
            top_surfaces: &[],
            chunk_slices: &[],
            chunk_perimeter_spacing: 0.0,
        },
    }
}

/// A single square slice builds a boundary whose contour keeps the layer's
/// shape inset by the variable-width inner offset.
#[test]
fn boundary_builds_from_layer_slices() {
    let layer = Box::leak(
        vec![ExPolygon::new(
            Polygon::new(vec![
                Point::new(-4_000_000, -4_000_000),
                Point::new(4_000_000, -4_000_000),
                Point::new(4_000_000, 4_000_000),
                Point::new(-4_000_000, 4_000_000),
            ]),
            Vec::new(),
        )]
        .into_boxed_slice(),
    );
    let geometry = LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: &[],
        support_islands: None,
        scale: CoordinateScale::Normal,
        previous_curled: &[],
        previous_layer_boundary: None,
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.42,
            layer_slices: layer,
            perimeter_spacing: 0.45,
            top_surfaces: &[],
            chunk_slices: &[],
            chunk_perimeter_spacing: 0.0,
        },
    };
    let super::boundary::BuildResult::Ready(boundary) = super::boundary::Boundary::build(
        &geometry.avoid_crossing,
        geometry.scale,
        [Point::new(0, 0); 2],
    )
    .unwrap() else {
        panic!("a square builds a boundary");
    };
    assert!(!boundary.contours.is_empty());
    let length: f64 = boundary
        .contours
        .iter()
        .map(|contour| {
            contour
                .windows(2)
                .map(|pair| {
                    (pair[1].x() as f64 - pair[0].x() as f64)
                        .hypot(pair[1].y() as f64 - pair[0].y() as f64)
                })
                .sum::<f64>()
        })
        .sum();
    // The 8 mm square inset by roughly 1.5 × spacing keeps most of its
    // perimeter (the variable offset shaves thin margins).
    assert!(length > 4.0 * 1_000_000.0, "perimeter {length}");
}
