//! The materialized flow for fill entities (role/bridge spacing rules).

use crate::ExtrusionRole;

// FillBase.cpp:148-157: propagate with_spacing failures at the source call site.
pub(in crate::project_slice) fn materialized_flow(
    params: crate::project_slice::group_fills::SurfaceFillParams,
    spacing: f32,
) -> Result<crate::project_slice::perimeters::types::Flow, crate::SliceError> {
    let mut flow = if params.extrusion_role == ExtrusionRole::InternalInfill && !params.bridge {
        params.flow
    } else {
        crate::project_slice::perimeters::flow::with_spacing(params.flow, spacing)?
    };
    // Bottom and internal-bridge flow are still pre-baked; top flow is resolved at export.
    flow.mm3_per_mm *= params.flow_ratio;
    Ok(flow)
}

/// `FillRectilinear::extended_object_bounding_box()` (FillRectilinear.cpp:2741-2750)
/// applied to the raw model-size object box. This helper centers it on
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
