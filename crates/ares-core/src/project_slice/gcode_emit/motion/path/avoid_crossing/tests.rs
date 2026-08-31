use super::{Request, route};
use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        gcode_emit::motion::{LayerGeometry, arc::Point as MotionPoint},
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
        internal_surfaces: Box::leak(vec![surface].into_boxed_slice()),
        scale: CoordinateScale::Normal,
        previous_layer_boundary: None,
    }
}

#[test]
fn source_rectangle_routes_crossing_travels_along_inset_boundary() {
    let geometry = geometry();
    let inset = 0.610_62;
    let offset = (110.0, 110.0);

    assert_eq!(
        route(Request {
            start: MotionPoint {
                x: 105.674,
                y: 117.189,
            },
            end: MotionPoint {
                x: 114.325,
                y: 114.325,
            },
            geometry,
            offset,
            inset,
            after_skirt: true,
        }),
        [MotionPoint {
            x: 114.389_38,
            y: 114.325,
        }]
    );
    assert_eq!(
        route(Request {
            start: MotionPoint {
                x: 113.775,
                y: 114.739,
            },
            end: MotionPoint {
                x: 112.604,
                y: 113.918,
            },
            geometry,
            offset,
            inset,
            after_skirt: false,
        }),
        [
            MotionPoint {
                x: 113.775,
                y: 114.389_38,
            },
            MotionPoint {
                x: 112.604,
                y: 114.389_38,
            },
        ]
    );
}
