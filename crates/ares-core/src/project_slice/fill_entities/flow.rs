//! The materialized flow for fill entities (role/bridge spacing rules).

use crate::ExtrusionRole;

pub(in crate::project_slice) fn materialized_flow(
    params: crate::project_slice::group_fills::SurfaceFillParams,
    spacing: f32,
) -> crate::project_slice::perimeters::types::Flow {
    let mut flow = if params.extrusion_role == ExtrusionRole::InternalInfill && !params.bridge {
        params.flow
    } else {
        crate::project_slice::perimeters::flow::with_spacing(params.flow, spacing)
    };
    flow.mm3_per_mm *= params.flow_ratio;
    flow
}

/// The print-object center used to anchor infill grids
/// (`Fill::_infill_direction`, `FillBase.cpp:288-290`).
pub(in crate::project_slice) fn object_center(
    object_slices: &[Vec<crate::geometry::ExPolygon>],
) -> crate::geometry::Point {
    let mut minimum_x = i64::MAX;
    let mut minimum_y = i64::MAX;
    let mut maximum_x = i64::MIN;
    let mut maximum_y = i64::MIN;
    for layer in object_slices {
        for expolygon in layer {
            for point in expolygon.contour().points() {
                minimum_x = minimum_x.min(point.x());
                minimum_y = minimum_y.min(point.y());
                maximum_x = maximum_x.max(point.x());
                maximum_y = maximum_y.max(point.y());
            }
        }
    }
    if minimum_x > maximum_x {
        return crate::geometry::Point::new(0, 0);
    }
    crate::geometry::Point::new(
        minimum_x + (maximum_x - minimum_x) / 2,
        minimum_y + (maximum_y - minimum_y) / 2,
    )
}

/// `Fill::extended_object_bounding_box()` (`FillBase.cpp:1569-1578`) applied
/// to the object bounding box. The incoming `bounding_box` is the slice-contour
/// extent (empirically this is what the oracle's grid anchors on — see
/// `fill_entities/generate.rs` and the residual map). This helper centers it on
/// the origin (matching `PrintObject::bounding_box`'s centered `m_size` box,
/// `Print.hpp:347`), merges the transposed corners
/// (`out.merge(min.y(),min.x())` + `out.merge(max.y(),max.x())`), then applies
/// `BoundingBox::scaled(sqrt(2))` (`BoundingBox.cpp:52-56` scales `min`/`max`
/// about the ORIGIN). Returns the scaled x-bounds used to anchor
/// consistent-pattern infill grids (`FillRectilinear.cpp:2785`).
pub(in crate::project_slice) fn extended_object_bounding_box_x(
    bounding_box: crate::geometry::BoundingBox,
) -> (i64, i64) {
    // `PrintObject::bounding_box()` — center the raw extent on the origin.
    let half_x = (bounding_box.max().x() - bounding_box.min().x()) / 2;
    let half_y = (bounding_box.max().y() - bounding_box.min().y()) / 2;
    let mut bounds = crate::geometry::BoundingBox::new(
        crate::geometry::Point::new(-half_x, -half_y),
        crate::geometry::Point::new(half_x, half_y),
    );
    bounds.merge_point(crate::geometry::Point::new(
        bounds.min().y(),
        bounds.min().x(),
    ));
    bounds.merge_point(crate::geometry::Point::new(
        bounds.max().y(),
        bounds.max().x(),
    ));
    (
        (bounds.min().x() as f64 * std::f64::consts::SQRT_2) as i64,
        (bounds.max().x() as f64 * std::f64::consts::SQRT_2) as i64,
    )
}
