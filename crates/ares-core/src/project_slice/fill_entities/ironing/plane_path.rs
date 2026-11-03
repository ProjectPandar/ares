use crate::{
    SliceError,
    fill::plane_path::{self, PlanePathPattern},
    geometry::{CoordinateScale, ExPolygon, Polyline},
};

pub(super) fn polylines(
    area: &ExPolygon,
    pattern: PlanePathPattern,
    spacing: f64,
    angle: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, SliceError> {
    // Fill.cpp:1634/1718: dispatch to the existing solid PlanePath fill.
    Ok(plane_path::fill_surface(
        area,
        pattern,
        plane_path::PlanePathFillParams {
            // Fill.cpp:1814-1816 leaves FillParams::extrusion_role at 0,
            // not erInternalInfill (FillBase.hpp:116).
            sparse: false,
            spacing,
            overlap: 0.0,
            density: 1.0,
            angle,
            multiline: 1,
            resolution: 0.0125,
            smooth_factor: 0.0,
            anchor_length: 1000.0,
            anchor_length_max: 1000.0,
            object_bounding_box: crate::geometry::BoundingBox::from_expolygon(area)
                .expect("ironing area has a contour"),
            calibration_order: false,
        },
        scale,
    )
    .map_err(super::geometry_error)?
    .into_iter()
    .flatten()
    .collect())
}
