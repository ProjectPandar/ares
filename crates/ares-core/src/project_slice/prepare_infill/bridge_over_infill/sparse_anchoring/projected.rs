//! Density projection and slice-contour bounds for sparse anchoring.

use crate::RegionOptions;

pub(in crate::project_slice) fn projected_sparse_density(options: &RegionOptions) -> f32 {
    let density_percent = options.sparse_infill_density.0 as f32;
    (0.01_f64 * f64::from(density_percent)) as f32
}

/// `object_bounding_box` for the plane-path fills — bounds over every slice
/// contour point (same construction as
/// `fill_entities::plane_path_object_bounding_box`).
pub(in crate::project_slice) fn object_bounding_box(
    object_slices: &[Vec<crate::geometry::ExPolygon>],
) -> crate::geometry::BoundingBox {
    let mut bb_points = object_slices
        .iter()
        .flatten()
        .flat_map(|expolygon| expolygon.contour().points());
    bb_points.next().map_or_else(
        || {
            crate::geometry::BoundingBox::new(
                crate::geometry::Point::new(0, 0),
                crate::geometry::Point::new(0, 0),
            )
        },
        |first| {
            bb_points.fold(
                crate::geometry::BoundingBox::new(*first, *first),
                |bb, point| {
                    crate::geometry::BoundingBox::new(
                        crate::geometry::Point::new(
                            bb.min().x().min(point.x()),
                            bb.min().y().min(point.y()),
                        ),
                        crate::geometry::Point::new(
                            bb.max().x().max(point.x()),
                            bb.max().y().max(point.y()),
                        ),
                    )
                },
            )
        },
    )
}
