//! Projection helpers for the sparse-infill anchor projection — density and
//! anchor-length resolution shared by the transaction gate and generation.

use crate::{FloatOrPercent, RegionOptions};

pub(in crate::project_slice) fn projected_sparse_density(options: &RegionOptions) -> f32 {
    let density_percent = options.sparse_infill_density.0 as f32;
    (0.01_f64 * f64::from(density_percent)) as f32
}

pub(in crate::project_slice) fn projected_anchor_lengths(
    options: &RegionOptions,
    spacing: f64,
) -> (f32, f32) {
    let anchor_length = projected_length(options.infill_anchor, spacing);
    let anchor_length_max = projected_length(options.infill_anchor_max, spacing);
    (anchor_length.min(anchor_length_max), anchor_length_max)
}

fn projected_length(value: FloatOrPercent, spacing: f64) -> f32 {
    match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(value) => (f64::from(value.0 as f32) * 0.01_f64 * spacing) as f32,
    }
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
